use wow_world_messages::wrath::CinematicSequenceId;

use crate::prelude::*;

impl super::Character {
    pub(super) async fn try_perform_first_time_login_if_required(&mut self) -> Result<()> {
        if self.needs_first_login {
            self.perform_first_login().await?;
            self.needs_first_login = false;
        }
        Ok(())
    }

    pub async fn perform_first_login(&mut self) -> Result<()> {
        assert!(self.needs_first_login);
        handlers::send_trigger_cinematic(self, CinematicSequenceId::BloodElf).await
    }
}
