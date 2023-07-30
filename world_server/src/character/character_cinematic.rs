use crate::prelude::*;
use wow_world_messages::wrath::CinematicSequenceId;

#[derive(Eq, PartialEq)]
pub(super) enum CharacterCinematicState {
    None,
    Queued(CinematicSequenceId),
    Watching(WatchingCinematicStateData),
}

#[derive(Eq, PartialEq)]
pub(super) struct WatchingCinematicStateData {
    pub cinematic_id: CinematicSequenceId,
    pub cinematic_spline_index: u32,
}

impl super::Character {
    pub async fn start_cinematic(&mut self, cinematic_id: CinematicSequenceId) -> Result<()> {
        if self.is_watching_cinematic() {
            bail!("Failed to start cinematic, character is already watching a cinematic");
        }

        self.cinematic_state = CharacterCinematicState::Queued(cinematic_id);
        handlers::send_trigger_cinematic(self, cinematic_id).await
    }
    #[allow(dead_code)]
    pub(super) async fn tick_cinematic(&mut self) -> Result<()> {
        //Advance the camera position, set "far sight" mode wherever it moves
        //So that creatures are spawned and visible inside the cinematic, etc
        Ok(())
    }

    pub fn handle_cinematic_next_camera(&mut self) -> Result<()> {
        //The client informs the server that it's ready move to the next camera spline.
        //Most cinematics have only one spline. It is also sent for the very first spline,
        //indicating the starting of the cinematic by the client

        if let CharacterCinematicState::Queued(cinematic_id) = self.cinematic_state {
            self.cinematic_state = CharacterCinematicState::Watching(WatchingCinematicStateData {
                cinematic_id,
                cinematic_spline_index: 0,
            });
        } else if let CharacterCinematicState::Watching(ref mut current_cinematic_state) = &mut self.cinematic_state {
            current_cinematic_state.cinematic_spline_index += 1;
        } else {
            bail!("Client confirmed starting of the cinematic, but no cinematic was queued by the server");
        }
        Ok(())
    }

    pub fn handle_cinematic_ended(&mut self) -> Result<()> {
        if let CharacterCinematicState::Watching(_cinematic_state) = &self.cinematic_state {
            self.cinematic_state = CharacterCinematicState::None;
        } else {
            bail!("Client confirmed ending of the cinematic, but no cinematic was known to be playing by the server");
        }
        Ok(())
    }

    pub fn is_watching_cinematic(&self) -> bool {
        self.cinematic_state != CharacterCinematicState::None
    }
}
