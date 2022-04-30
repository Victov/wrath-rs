use crate::data::MovementFlags;
use crate::handlers::login_handler::{LogoutResult, LogoutSpeed};
use crate::handlers::{login_handler::LogoutState, movement_handler::TeleportationState};
use crate::prelude::*;
use crate::world::prelude::*;
use std::sync::Arc;

impl super::Character {
    pub(super) async fn tick_logout_state(&mut self, delta_time: f32, world: Arc<World>) -> Result<()> {
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
        //TODO: add checks about being in rested area (instant logout), being in combat (refuse), etc

        if self
            .movement_info
            .has_any_movement_flag(&[MovementFlags::Falling, MovementFlags::FallingFar])
        {
            return Ok((LogoutResult::FailJumpingOrFalling, LogoutSpeed::Delayed));
        }

        let delayed = true;
        Ok(match self.logout_state {
            LogoutState::None if delayed => {
                self.logout_state = LogoutState::Pending(std::time::Duration::from_secs(20));
                self.set_stunned(true)?;
                self.set_rooted(true).await?;
                self.set_character_stand_state(stand_state::UnitStandState::Sit).await?;
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
            self.set_stunned(false)?;
            self.set_rooted(false).await?;
            self.set_character_stand_state(stand_state::UnitStandState::Stand).await?;
            self.logout_state = LogoutState::None;
        } else {
        }
        Ok(())
    }

    //This function will trigger every tick as long as the state is LogoutState::Executing
    async fn execute_logout(&mut self, world: Arc<World>) -> Result<()> {
        if self.teleportation_state != TeleportationState::None {
            return Ok(());
        }

        world
            .get_instance_manager()
            .try_get_map_for_character(self)
            .await
            .ok_or_else(|| anyhow!("Invalid map during logout"))?
            .remove_object_by_guid(&self.guid)
            .await;

        handlers::send_smsg_logout_complete(self).await?;

        self.logout_state = LogoutState::ReturnToCharSelect;

        Ok(())
    }
}
