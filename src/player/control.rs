use std::str::FromStr;

use anyhow::Result;
use strum_macros::EnumString;

use super::waywallen;

#[derive(EnumString)]
pub enum ControlAction {
    Next,
    Prev,
    Reload,
    Stop,
    Continue,
    Pause,
}

pub fn control(action: Option<&str>) -> Result<()> {
    let Some(action) = action else {
        return Ok(());
    };
    let action = ControlAction::from_str(action)
        .map_err(|error| anyhow::anyhow!("Invalid control action {action}: {error}"))?;

    match action {
        ControlAction::Next => waywallen::next(),
        ControlAction::Prev => waywallen::previous(),
        ControlAction::Reload => waywallen::restart(),
        ControlAction::Pause => waywallen::toggle_pause(),
        ControlAction::Stop | ControlAction::Continue => {
            todo!("Waywallen playlist activation control is not implemented")
        }
    }
}
