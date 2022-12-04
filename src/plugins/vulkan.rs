use std::sync::Arc;

use bevy::app::{App, Plugin};
use vulkano::pipeline::graphics::viewport::Viewport;

pub use config::get_vulkano_config;
use systems::*;
use systems::create_pipelines;

use super::super::plugins::ViewportResource;

mod systems;
pub mod resources;
mod config;

pub struct VulkanPlugin {}

impl VulkanPlugin {
    pub fn default() -> Self {
        Self {}
    }
}

impl Plugin for VulkanPlugin {
    fn build(&self, app: &mut App) {
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [1280.0, 800.0],
            depth_range: 0.0..1.0,
        };
        app
            .insert_resource(ViewportResource { viewport: Arc::new(viewport) })
            .add_startup_system(create_pipelines)
            .add_system(render);
    }

    fn name(&self) -> &str {
        "Vulkan_plugin"
    }

    fn is_unique(&self) -> bool {
        true
    }
}