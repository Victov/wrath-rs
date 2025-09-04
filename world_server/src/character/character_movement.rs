use crate::data::{PositionAndOrientation, WorldZoneLocation};
use crate::handlers::movement_handler::{TeleportationDistance, TeleportationState};
use crate::prelude::*;
use crate::world::{game_object::GameObject, World};
use wow_world_messages::wrath::{MovementInfo, MovementInfo_MovementFlags};

impl super::Character {
    pub fn process_movement(&mut self, movement_info: MovementInfo) {
        self.movement_info = movement_info;
    }

    pub fn set_position(&mut self, position: &PositionAndOrientation) {
        self.movement_info.position = position.position;
        self.movement_info.orientation = position.orientation;
    }

    fn reset_move_flags(&mut self) {
        self.movement_info.flags = MovementInfo_MovementFlags::empty();
    }

    pub fn teleport_to(&mut self, destination: TeleportationDistance) {
        self.teleportation_state = TeleportationState::Queued(destination);
    }

    pub(super) async fn handle_queued_teleport(&mut self, world: &mut World) -> Result<()> {
        //TODO: Handle the possibility that the player may have logged out
        //between queuing and handling the teleport

        let state = self.teleportation_state.clone();
        match state {
            TeleportationState::Queued(TeleportationDistance::Near(dest)) => self.execute_near_teleport(dest.clone()).await?,
            TeleportationState::Queued(TeleportationDistance::Far(dest)) => self.execute_far_teleport(dest.clone(), world).await?,
            _ => {}
        };

        Ok(())
    }

    async fn execute_near_teleport(&mut self, destination: PositionAndOrientation) -> Result<()> {
        //The rest of the teleportation is handled when the client sends back this packet
        self.teleportation_state = TeleportationState::Executing(TeleportationDistance::Near(destination.clone()));

        handlers::send_msg_move_teleport_ack(self, &destination).await?;
        Ok(())
    }

    async fn execute_far_teleport(&mut self, destination: WorldZoneLocation, world: &mut World) -> Result<()> {
        if self.map == destination.map {
            //This was not actually a far teleport. It should have been a near teleport since we're
            //on the same map.
            self.teleport_to(TeleportationDistance::Near(destination.into()));
            return Ok(());
        }

        handlers::send_smsg_transfer_pending(self, destination.map).await?;
        self.reset_move_flags();

        let old_map = world
            .get_instance_manager_mut()
            .try_get_map_for_character_mut(self)
            .ok_or_else(|| anyhow!("Player is teleporting away from an invalid map"))?;

        old_map.remove_object_by_guid(self.get_guid());

        let wzl = destination.clone().into();
        handlers::send_smsg_new_world(self, destination.map, wzl).await?;

        self.teleportation_state = TeleportationState::Executing(TeleportationDistance::Far(destination));
        Ok(())
    }
}
