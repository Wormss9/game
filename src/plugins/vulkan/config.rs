use std::borrow::Borrow;
use std::sync::Arc;

use bevy::log::{error, info, trace, warn};
use bevy_vulkano::VulkanoWinitConfig;
use vulkano::{device, instance, Version, VulkanLibrary};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::instance::debug::{self, ValidationFeatureEnable::{DebugPrintf, SynchronizationValidation}};

//BestPractices

pub fn get_vulkano_config() -> VulkanoWinitConfig {
    let library = VulkanLibrary::new().unwrap_or_else(|err| panic!("Couldn't load Vulkan library: {:?}", err));
    let enabled_extensions = instance::InstanceExtensions {
        ext_validation_features: cfg!(debug_assertions),
        ext_debug_utils: cfg!(debug_assertions),
        ..vulkano_win::required_extensions(library.borrow())
    };
    let enabled_layers = if cfg!(debug_assertions) { vec!["VK_LAYER_KHRONOS_validation".to_string()] } else { vec![] };
    let enabled_validation_features = if cfg!(debug_assertions) { vec![DebugPrintf, SynchronizationValidation] } else { vec![] };

    let instance_create_info = instance::InstanceCreateInfo {
        application_name: Some("Gamess9 Game".to_string()),
        application_version: Version { major: 0, minor: 0, patch: 1 },
        enabled_extensions,
        enabled_layers,
        engine_name: Some("Gamess9 Engine".to_string()),
        engine_version: Version { major: 0, minor: 0, patch: 1 },
        enabled_validation_features,
        ..Default::default()
    };

    let debug_create_info = if cfg!(debug_assertions) {
        info!("Creating debugger");
        let mut debug_create_info = debug::DebugUtilsMessengerCreateInfo::user_callback(Arc::new(debugger_callback));
        debug_create_info.message_severity =
            debug::DebugUtilsMessageSeverity
            {
                error: true,
                warning: true,
                information: true,
                verbose: true,
                ..Default::default()
            };
        debug_create_info.message_type = debug::DebugUtilsMessageType {
            general: true,
            validation: true,
            performance: true,
            ..Default::default()
        };
        Some(debug_create_info)
    } else { None };

    let device_extensions = device::DeviceExtensions {
        khr_swapchain: true,
        ..device::DeviceExtensions::empty()
    };

    let device_features = device::Features {
        ..device::Features::empty()
    };

    let vulkano_config = vulkano_util::context::VulkanoConfig {
        instance_create_info,
        debug_create_info,
        device_filter_fn: Arc::new(device_filter_fn),
        device_priority_fn: Arc::new(device_priority_fn),
        device_extensions,
        device_features,
        print_device_name: cfg!(debug_assertions),
    };

    VulkanoWinitConfig {
        return_from_run: false,
        vulkano_config,
        is_gui_overlay: true,
        add_primary_window: true,
    }
}

fn device_priority_fn(device: &PhysicalDevice) -> u32 {
    if !device.supported_features().geometry_shader {
        return 0;
    }
    let properties = device.properties();
    let mut score = 0;

    if properties.device_type == PhysicalDeviceType::DiscreteGpu {
        score += 1000;
    }

    u32::MAX - score - properties.max_image_dimension2_d
}

fn device_filter_fn(_: &PhysicalDevice) -> bool { true }

fn debugger_callback(message: &debug::Message) {
    let ty = if message.ty.general { "gen" } else if message.ty.validation { "val" } else { "per" };
    let prefix = match message.layer_prefix {
        Some(layer) => layer,
        None => "Unknown",
    };
    let response = format!("[{:}][{:}] {:}", ty, prefix, message.description);

    if message.severity.error { error!("{}",response) } else
    if message.severity.warning { warn!("{}",response) } else
    if message.severity.information { info!("{}",response) } else { trace!("{}",response) };
}