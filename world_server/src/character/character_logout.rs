use crate::handlers::{login_handler::LogoutState, movement_handler::TeleportationState};
use crate::prelude::*;
use crate::world::prelude::*;
use wow_world_messages::wrath::{LogoutResult, LogoutSpeed, UnitStandState};

impl super::Character {
    pub(super) async fn tick_logout_state(&mut self, delta_time: f32, world: &mut World) -> Result<()> {
        match &mut self.logout_state {
            LogoutState::Pending(duration_left) => {
                *duration_left = duration_left.saturating_sub(std::time::Duration::from_secs_f32(delta_time));
                if duration_left.is_zero() {
                    self.logout_state = LogoutState::Executing;
                }
                Ok(())
            }
            LogoutState::Executing => self.execute_logout(world).await,
            _ => Ok(()),
        }
    }

    pub async fn try_logout(&mut self) -> Result<(LogoutResult, LogoutSpeed)> {
        //TODO: add checks about being in combat (refuse), etc

        if self.movement_info.flags.get_falling_far() || self.movement_info.flags.get_falling_slow() {
            return Ok((LogoutResult::FailureJumpingOrFalling, LogoutSpeed::Delayed));
        }

        let delayed = !self.is_in_rested_area();
        Ok(match self.logout_state {
            LogoutState::None if delayed => {
                self.logout_state = LogoutState::Pending(std::time::Duration::from_secs(20));
                self.set_stunned(true);
                self.set_rooted(true).await?;
                self.set_stand_state(UnitStandState::Sit).await?;
                (LogoutResult::Success, LogoutSpeed::Delayed)
            }
            LogoutState::None if !delayed => {
                self.logout_state = LogoutState::Executing;
                (LogoutResult::Success, LogoutSpeed::Instant)
            }
            _ => (LogoutResult::Success, LogoutSpeed::Instant),
        })
    }

    pub async fn cancel_logout(&mut self) -> Result<()> {
        if let LogoutState::Pending(_) = self.logout_state {
            self.set_stunned(false);
            self.set_rooted(false).await?;
            self.set_stand_state(UnitStandState::Stand).await?;
            self.logout_state = LogoutState::None;
            Ok(())
        } else {
            bail!("Cancelling logout, but no logout is in progress")
        }
    }

    //This function will trigger every tick as long as the state is LogoutState::Executing
    async fn execute_logout(&mut self, world: &mut World) -> Result<()> {
        if self.teleportation_state != TeleportationState::None {
            return Ok(());
        }

        world
            .get_instance_manager_mut()
            .try_get_map_for_character_mut(self)
            .ok_or_else(|| anyhow!("Invalid map during logout"))?
            .remove_object_by_guid(self.get_guid());

        handlers::send_smsg_logout_complete(self).await?;

        self.logout_state = LogoutState::ReturnToCharSelect;

        Ok(())
    }
}
