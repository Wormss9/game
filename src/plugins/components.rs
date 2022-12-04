use bevy::app::{App, Plugin};

pub use components::*;

mod components;

pub struct Components {}

impl Components {
    pub fn default() -> Self {
        Self {}
    }
}

impl Plugin for Components {
    fn build(&self, app: &mut App) {
        app.register_type::<Body>()
            .register_type::<Name>()
            .register_type::<Vertebrae>()
            .register_type::<Vec<Vertebrae>>()
            .register_type::<[f32; 2]>()
            .register_type::<[f32; 4]>();
    }

    fn name(&self) -> &str {
        "Components_system"
    }

    fn is_unique(&self) -> bool {
        true
    }
}