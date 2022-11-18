use std::borrow::Borrow;
use std::sync::Arc;

use vulkano::{instance::{Instance, InstanceCreateInfo, InstanceExtensions}, swapchain, sync, Version, VulkanLibrary};
use vulkano::buffer::{BufferAccess, BufferContents, BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Features, physical::{PhysicalDevice, PhysicalDeviceType}, Queue, QueueCreateInfo};
use vulkano::format::Format;
use vulkano::image::{ImageAccess, ImageLayout, ImageUsage, SwapchainImage};
use vulkano::image::view::ImageView;
use vulkano::instance::debug::{DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger, DebugUtilsMessengerCreateInfo, Message};
use vulkano::instance::debug::ValidationFeatureEnable::*;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::input_assembly;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::{VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate, VertexInputState};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{AttachmentDescription, AttachmentReference, Framebuffer, FramebufferCreateInfo, LoadOp, RenderPass, RenderPassCreateInfo, StoreOp, Subpass, SubpassDescription};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{acquire_next_image, AcquireError, Surface, SurfaceInfo, Swapchain, SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo};
use vulkano::sync::{FlushError, GpuFuture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use crate::Test;
use crate::vertex::Vertex;

pub struct Vulkan
{
    window: Arc<Window>,
    device: Arc<Device>,
    queues: (Arc<Queue>, Arc<Queue>),
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
    graphics_pipeline: Arc<GraphicsPipeline>,
    frame_buffers: Vec<Arc<Framebuffer>>,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
}

impl Vulkan
{
    pub fn new(event_loop: &EventLoop<()>, data: [Vertex; 4]) -> Self {
        let window = Self::create_window(event_loop);
        let instance = Self::create_instance();
        let _debugger = Self::create_debugger(instance.clone());
        let surface = Self::create_surface(instance.clone(), window.clone());
        let physical_device = Self::get_physical_device(instance.clone());
        let (device, queues) = Self::get_device(physical_device.clone());
        let (swapchain, swapchain_images) = Self::create_swapchain(physical_device.clone(), device.clone(), surface.clone());
        let render_pass = Self::create_render_pass(device.clone());
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: window.inner_size().into(),
            depth_range: 0.0..1.0,
        };
        let graphics_pipeline = Self::create_pipeline(device.clone(), render_pass.clone(), viewport.clone());
        let frame_buffers = Self::create_frame_buffers(render_pass.clone(), &swapchain_images);
        let previous_frame_end = Some(sync::now(device.clone()).boxed());
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            &memory_allocator,
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            data,
        ).expect("Failed to create vertex buffer.");
        println!("{:}", vertex_buffer.size());


        Self {
            window,
            device,
            queues,
            swapchain,
            render_pass,
            viewport,
            graphics_pipeline,
            frame_buffers,
            recreate_swapchain: false,
            previous_frame_end,
            memory_allocator,
            vertex_buffer,
        }
    }
    fn create_window(event_loop: &EventLoop<()>) -> Arc<Window> {
        Arc::new(WindowBuilder::new()
            .with_inner_size(LogicalSize::new(1280, 800))
            .with_title("Gamess9 Game")
            .with_fullscreen(None)
            .build(&event_loop)
            .expect("Failed to create window"))
    }
    fn create_instance() -> Arc<Instance> {
        let library = VulkanLibrary::new().unwrap_or_else(|err| panic!("Couldn't load Vulkan library: {:?}", err));
        let enabled_extensions = InstanceExtensions {
            ext_validation_features: cfg!(debug_assertions),
            ext_debug_utils: cfg!(debug_assertions),
            ..vulkano_win::required_extensions(library.borrow())
        };
        let mut enabled_layers = vec![];
        if cfg!(debug_assertions) { enabled_layers.push("VK_LAYER_KHRONOS_validation".to_string()) };
        let enabled_validation_features = if cfg!(debug_assertions) { vec![DebugPrintf, BestPractices, SynchronizationValidation] } else { vec![] };
        let instance_create_info = InstanceCreateInfo {
            application_name: Some("Gamess9 Game".to_string()),
            application_version: Version { major: 0, minor: 0, patch: 1 },
            enabled_extensions,
            enabled_layers,
            engine_name: Some("Gamess9 Engine".to_string()),
            engine_version: Version { major: 0, minor: 0, patch: 1 },
            enabled_validation_features,
            ..Default::default()
        };
        Instance::new(
            library,
            instance_create_info,
        ).unwrap()
    }
    fn create_debugger(instance: Arc<Instance>) -> Option<DebugUtilsMessenger> {
        if !cfg!(debug_assertions) { return None; };
        let mut debug_create_info = DebugUtilsMessengerCreateInfo::user_callback(Arc::new(debugger_callback));
        debug_create_info.message_severity =
            DebugUtilsMessageSeverity
            {
                error: true,
                warning: true,
                information: false,
                verbose: true,
                ..Default::default()
            };
        debug_create_info.message_type = DebugUtilsMessageType {
            general: true,
            validation: true,
            performance: true,
            ..Default::default()
        };
        println!("Debugger created");
        Some(unsafe {
            DebugUtilsMessenger::new(
                instance,
                debug_create_info,
            )
        }.expect("Failed to create debugger."))
    }
    fn get_physical_device(instance: Arc<Instance>) -> Arc<PhysicalDevice> {
        let physical_devices = instance.enumerate_physical_devices().unwrap();
        let mut candidates: Vec<(u32, Arc<PhysicalDevice>)> = Vec::new();

        for physical_device in physical_devices {
            let score = rate_device_suitability(physical_device.clone());
            candidates.push((score, physical_device));
        }

        let (_, physical_device) = candidates
            .iter()
            .max_by(|a, b| a.0.cmp(&b.0))
            .unwrap()
            .to_owned();

        physical_device
    }
    fn get_device(physical_device: Arc<PhysicalDevice>) -> (Arc<Device>, (Arc<Queue>, Arc<Queue>)) {
        let queue_indices = Self::get_queue_indices(physical_device.clone());
        let enabled_features = Features {
            geometry_shader: true,
            ..Features::empty()
        };
        let enabled_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };
        let queue_create_infos = vec![
            QueueCreateInfo {
                queue_family_index: queue_indices.0,
                ..Default::default()
            },
            QueueCreateInfo {
                queue_family_index: queue_indices.1,
                ..Default::default()
            }];
        let create_info = DeviceCreateInfo {
            enabled_extensions,
            enabled_features,
            queue_create_infos,
            ..Default::default()
        };
        let (device, mut queues) = Device::new(physical_device,
                                               create_info,
        ).expect("Failed to create device");

        (device,
         (queues.nth(0).expect("Couldn't get graphical queue"),
          queues.nth(0).expect("Couldn't get transfer queue")))
    }
    fn get_queue_indices(physical_device: Arc<PhysicalDevice>) -> (u32, u32) {
        let mut found_graphics_q_index = None;
        let mut found_transfer_q_index = None;
        for (index, q_fam) in physical_device.queue_family_properties().iter().enumerate() {
            if q_fam.queue_count > 0 && q_fam.queue_flags.graphics {
                found_graphics_q_index = Some(index as u32);
            }
            if q_fam.queue_count > 0
                && q_fam.queue_flags.transfer
                && (found_transfer_q_index.is_none()
                || !q_fam.queue_flags.graphics)
            {
                found_transfer_q_index = Some(index as u32);
            }
        }
        (
            found_graphics_q_index.expect("Graphics queue not found"),
            found_transfer_q_index.expect("Transfer queue not found"),
        )
    }
    fn create_surface(instance: Arc<Instance>, window: Arc<Window>) -> Arc<Surface> {
        vulkano_win::create_surface_from_winit(window, instance).expect("Failed to create surface")
    }
    fn create_swapchain(physical_device: Arc<PhysicalDevice>, device: Arc<Device>, surface: Arc<Surface>) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
        let surface_capabilities = physical_device.surface_capabilities(surface.borrow(), SurfaceInfo::default()).expect("Failed to get surface capabilities");

        let image_usage = ImageUsage {
            color_attachment: true,
            ..ImageUsage::default()
        };

        let create_info = SwapchainCreateInfo {
            min_image_count: 3.max(surface_capabilities.min_image_count)
                .min(surface_capabilities.max_image_count.unwrap()),
            image_usage,
            present_mode: swapchain::PresentMode::Immediate,
            ..Default::default()
        };
        Swapchain::new(device, surface, create_info).expect("Failed to create swapchain")
    }
    fn create_render_pass(device: Arc<Device>) -> Arc<RenderPass> {
        let attachment_description = AttachmentDescription {
            format: Some(Format::B8G8R8A8_UNORM),
            load_op: LoadOp::Clear,
            store_op: StoreOp::Store,
            initial_layout: ImageLayout::General,
            final_layout: ImageLayout::PresentSrc,
            ..Default::default()
        };

        let color_attachment_reference = AttachmentReference {
            attachment: 0,
            layout: ImageLayout::ColorAttachmentOptimal,
            ..AttachmentReference::default()
        };
        let subpass_description = SubpassDescription {
            color_attachments: vec![Some(color_attachment_reference)],
            ..SubpassDescription::default()
        };

        let create_info = RenderPassCreateInfo {
            attachments: vec![attachment_description],
            subpasses: vec![subpass_description],
            ..Default::default()
        };
        RenderPass::new(
            device,
            create_info,
        ).expect("Failed to create render pass")
    }
    fn create_pipeline(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        viewport: Viewport,
    ) -> Arc<GraphicsPipeline> {
        let vertex_shader = unsafe {
            ShaderModule::from_words(
                device.clone(),
                vk_shader_macros::include_glsl!("./src/shaders/shader.vert", kind: vert),
            )
        }.expect("Failed to create vertex shader module");
        let fragment_shader = unsafe {
            ShaderModule::from_words(
                device.clone(),
                vk_shader_macros::include_glsl!("./src/shaders/shader.frag"),
            )
        }.expect("Failed to create fragment shader module");

        let input_assembly_state = InputAssemblyState::new()
            .topology(input_assembly::PrimitiveTopology::PointList);

        let vertex_input_state = VertexInputState::default()
            .binding(0, VertexInputBindingDescription {
                stride: 16,
                input_rate: VertexInputRate::Vertex,
            })
            .attribute(0, VertexInputAttributeDescription {
                binding: 0,
                format: Format::R32G32B32A32_SFLOAT,
                offset: 0,
            });

        GraphicsPipeline::start()
            //.vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
            .vertex_input_state(vertex_input_state)
            .input_assembly_state(input_assembly_state)
            .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
            .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
            .render_pass(Subpass::from(render_pass, 0).unwrap())
            .build(device)
            .unwrap()
    }
    fn create_frame_buffers(render_pass: Arc<RenderPass>, images: &Vec<Arc<SwapchainImage>>) -> Vec<Arc<Framebuffer>> {
        let mut framebuffers = vec![];

        for image in images {
            let view = ImageView::new_default(image.clone()).unwrap();
            let create_info = FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            };
            let framebuffer = Framebuffer::new(
                render_pass.clone(),
                create_info,
            ).unwrap();
            framebuffers.push(framebuffer);
        };
        framebuffers
    }
    pub fn set_vertexes(&self, data: [Vertex; 4]) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
        let memory_allocator: &StandardMemoryAllocator = self.memory_allocator.borrow();
        CpuAccessibleBuffer::from_iter(
            memory_allocator,
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            data,
        ).expect("Failed to create vertex buffer.")
    }
    pub fn event_handler(&mut self, event: Event<()>, control_flow: &mut ControlFlow, test: &mut Test) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                self.recreate_swapchain = true;
            }
            Event::RedrawEventsCleared => {
                // Do not draw frame when screen dimensions are zero.
                // On Windows, this can occur from minimizing the application.
                let dimensions = self.window.inner_size();
                if dimensions.width == 0 || dimensions.height == 0 {
                    return;
                }

                // It is important to call this function from time to time, otherwise resources will keep
                // accumulating and you will eventually reach an out of memory error.
                // Calling this function polls various fences in order to determine what the GPU has
                // already processed, and frees the resources that are no longer needed.
                self.previous_frame_end.as_mut().unwrap().cleanup_finished();

                // Whenever the window resizes we need to recreate everything dependent on the window size.
                // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
                if self.recreate_swapchain {
                    // Use the new dimensions of the window.

                    let (new_swapchain, new_images) =
                        match self.swapchain.recreate(SwapchainCreateInfo {
                            image_extent: dimensions.into(),
                            ..self.swapchain.create_info()
                        }) {
                            Ok(r) => r,
                            // This error tends to happen when the user is manually resizing the window.
                            // Simply restarting the loop is the easiest way to fix this issue.
                            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };

                    self.swapchain = new_swapchain;
                    // Because framebuffers contains an Arc on the old swapchain, we need to
                    // recreate framebuffers as well.
                    self.frame_buffers = self.window_size_dependent_setup(
                        &new_images,
                    );
                    self.recreate_swapchain = false;
                }

                // Before we can draw on the output, we have to *acquire* an image from the swapchain. If
                // no image is available (which happens if you submit draw commands too quickly), then the
                // function will block.
                // This operation returns the index of the image that we are allowed to draw upon.
                //
                // This function can block if no image is available. The parameter is an optional timeout
                // after which the function call will return an error.
                let (image_index, suboptimal, acquire_future) =
                    match acquire_next_image(self.swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            self.recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                // acquire_next_image can be successful, but suboptimal. This means that the swapchain image
                // will still work, but it may not display correctly. With some drivers this can be when
                // the window resizes, but it may not cause the swapchain to become out of date.
                if suboptimal {
                    self.recreate_swapchain = true;
                }

                // In order to draw, we have to build a *command buffer*. The command buffer object holds
                // the list of commands that are going to be executed.
                //
                // Building a command buffer is an expensive operation (usually a few hundred
                // microseconds), but it is known to be a hot path in the driver and is expected to be
                // optimized.
                //
                // Note that we have to pass a queue family when we create the command buffer. The command
                // buffer will only be executable on that given queue family.
                let command_buffer_allocator =
                    StandardCommandBufferAllocator::new(self.device.clone(), Default::default());
                let mut builder = AutoCommandBufferBuilder::primary(
                    &command_buffer_allocator,
                    self.queues.0.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                    .unwrap();

                let vertex_buffer = self.set_vertexes(test.get());

                builder
                    // Before we can draw, we have to *enter a render pass*.
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            // A list of values to clear the attachments with. This list contains
                            // one item for each attachment in the render pass. In this case,
                            // there is only one attachment, and we clear it with a blue color.
                            //
                            // Only attachments that have `LoadOp::Clear` are provided with clear
                            // values, any others should use `ClearValue::None` as the clear value.
                            clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                            ..RenderPassBeginInfo::framebuffer(
                                self.frame_buffers[image_index as usize].clone(),
                            )
                        },
                        // The contents of the first (and only) subpass. This can be either
                        // `Inline` or `SecondaryCommandBuffers`. The latter is a bit more advanced
                        // and is not covered here.
                        SubpassContents::Inline,
                    )
                    .unwrap()
                    // We are now inside the first subpass of the render pass. We add a draw command.
                    //
                    // The last two parameters contain the list of resources to pass to the shaders.
                    // Since we used an `EmptyPipeline` object, the objects have to be `()`.
                    .set_viewport(0, [self.viewport.clone()])
                    .bind_pipeline_graphics(self.graphics_pipeline.clone())
                    .bind_vertex_buffers(0, vertex_buffer.clone())
                    .draw(vertex_buffer.len() as u32, 1, 0, 0)
                    .unwrap()
                    // We leave the render pass. Note that if we had multiple
                    // subpasses we could have called `next_subpass` to jump to the next subpass.
                    .end_render_pass()
                    .unwrap();

                // Finish building the command buffer by calling `build`.
                let command_buffer = builder.build().unwrap();

                let future = self.previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(self.queues.0.clone(), command_buffer)
                    .unwrap()
                    // The color output is now expected to contain our triangle. But in order to show it on
                    // the screen, we have to *present* the image by calling `present`.
                    //
                    // This function does not actually present the image immediately. Instead it submits a
                    // present command at the end of the queue. This means that it will only be presented once
                    // the GPU has finished executing the command buffer that draws the triangle.
                    .then_swapchain_present(
                        self.queues.0.clone(),
                        SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
                    )
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        self.previous_frame_end = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        self.recreate_swapchain = true;
                        self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
                    }
                    Err(e) => {
                        panic!("Failed to flush future: {:?}", e);
                        // previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                }
            }
            _ => (),
        }
    }
    pub fn window_size_dependent_setup(
        &mut self,
        images: &[Arc<SwapchainImage>],
    ) -> Vec<Arc<Framebuffer>> {
        let dimensions = images[0].dimensions().width_height();
        self.viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

        images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    self.render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                )
                    .unwrap()
            })
            .collect::<Vec<_>>()
    }
}

fn rate_device_suitability(device: Arc<PhysicalDevice>) -> u32 {
    if !device.supported_features().geometry_shader {
        return 0;
    }
    let properties = device.properties();
    let mut score = 0;

    if properties.device_type == PhysicalDeviceType::DiscreteGpu {
        score += 1000;
    }

    score + properties.max_image_dimension2_d
}

fn debugger_callback(message: &Message) {
    let severity = if message.severity.error { "error" } else if message.severity.warning { "warn" } else if message.severity.information { "info" } else { "verb" };
    let ty = if message.ty.general { "gen" } else if message.ty.validation { "val" } else { "per" };
    let prefix = match message.layer_prefix {
        Some(layer) => layer,
        None => "Unknown",
    };
    println!("[{:}][{:}][{:}] {:}", severity, ty, prefix, message.description);
}