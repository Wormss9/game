use bevy::*;
use bevy_ecs::system::Commands;

use plugins::components::*;

mod plugins;

fn main() {
    let window_descriptor = window::WindowDescriptor {
        width: 1280.0,
        height: 800.0,
        title: "Gamess9 Game".to_string(),
        mode: window::WindowMode::Windowed,
        ..Default::default()
    };

    app::App::new().add_plugin(core::CorePlugin::default())
        .insert_non_send_resource(plugins::get_vulkano_config())
        .add_plugin(log::LogPlugin::default())
        .add_plugin(input::InputPlugin::default())
        .add_plugin(time::TimePlugin::default())
        .add_plugin(asset::AssetPlugin::default())
        .add_plugin(scene::ScenePlugin::default())
        .add_plugin(bevy_vulkano::VulkanoWinitPlugin { window_descriptor })
        .add_plugin(plugins::VulkanPlugin::default())
        .add_plugin(plugins::Components::default())
        .add_plugin(plugins::SaveLoad::default())
        .add_startup_system(add_entities)
        .run();
}

fn add_entities(mut commands: Commands) {
    let vertebrae1 = Vertebrae { position: [0.0, 0.0, 0.0, 0.0], color: [0.0, 1.0, 0.0, 1.0] };
    let vertebrae2 = Vertebrae { position: [0.1, 0.1, 0.0, 0.0], color: [1.0, 1.0, 0.0, 1.0] };
    let vertebrae3 = Vertebrae { position: [0.5, 0.0, 0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] };
    commands.spawn((Body { spine: vec![vertebrae1, vertebrae2, vertebrae3] }, Save::default()));
    let vertebrae = Vertebrae { position: [-0.5, -0.5, 0.0, 0.0], color: [0.0, 0.0, 1.0, 1.0] };
    commands.spawn((Body { spine: vec![vertebrae] }, Name("TestWithSpine".to_string()), Save::default()));
    commands.spawn((Name("SpinelessOne".to_string()), Save::default()));
}

