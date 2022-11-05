use crate::{prelude::*, world::prelude::cinematic::get_opening_cinematic_for_race_class};

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
        if let Some(cinematic_id) = get_opening_cinematic_for_race_class(&self.get_race(), &self.get_class()) {
            self.start_cinematic(cinematic_id).await?;
        }
        Ok(())
    }
}
