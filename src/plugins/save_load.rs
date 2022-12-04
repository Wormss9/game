use bevy::app::{App, Plugin};

use systems::*;

pub mod systems;

pub struct SaveLoad {}

impl SaveLoad {
    pub fn default() -> Self {
        Self {}
    }
}

impl Plugin for SaveLoad {
    fn build(&self, app: &mut App) {
        app.add_system(quick_save_game)
            .add_system(quick_load_game);
    }

    fn name(&self) -> &str {
        "Save_Load"
    }

    fn is_unique(&self) -> bool {
        true
    }
}