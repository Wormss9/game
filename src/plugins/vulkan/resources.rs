use std::sync::Arc;

use bevy::ecs::system::Resource;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::buffer::TypedBufferAccess;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassContents};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::{self, Device, DeviceOwned};
use vulkano::format::Format;
use vulkano::image::{ImageLayout, ImageUsage, StorageImage};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::input_assembly;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::{VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate, VertexInputState};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{AttachmentDescription, AttachmentReference, Framebuffer, FramebufferCreateInfo, LoadOp, RenderPass, RenderPassCreateInfo, StoreOp, Subpass, SubpassDescription};
use vulkano::swapchain::AcquireError;
use vulkano::sync::GpuFuture;
use vulkano_util::renderer::{DeviceImageView, SwapchainImageView, VulkanoWindowRenderer};

#[derive(Resource)]
pub struct VulkanPipeline {
    queue: Arc<device::Queue>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
    graphics_pipeline: Arc<GraphicsPipeline>,
    render_pass: Arc<RenderPass>,
    allocator: Arc<StandardMemoryAllocator>,
    image: DeviceImageView,
}

impl VulkanPipeline {
    pub fn new(allocator: Arc<StandardMemoryAllocator>,
               window_renderer: &VulkanoWindowRenderer,
               viewport: Viewport) -> Self {
        let queue = window_renderer.graphics_queue().clone();
        let device = queue.device().clone();
        let render_pass = Self::create_render_pass(device.clone());
        let vertex_input_state = VertexInputState::default()
            .binding(0, VertexInputBindingDescription {
                stride: 32,
                input_rate: VertexInputRate::Vertex,
            })
            .attribute(0, VertexInputAttributeDescription {
                binding: 0,
                format: Format::R32G32B32A32_SFLOAT,
                offset: 0,
            })
            .attribute(1, VertexInputAttributeDescription {
                binding: 0,
                format: Format::R32G32B32A32_SFLOAT,
                offset: 16,
            });
        let graphics_pipeline = {
            let vertex_shader = vs::load(device.clone()).unwrap();
            let fragment_shader = fs::load(device.clone()).unwrap();

            let input_assembly_state = InputAssemblyState::new()
                .topology(input_assembly::PrimitiveTopology::PointList);

            GraphicsPipeline::start()
                .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
                .vertex_input_state(vertex_input_state)
                .input_assembly_state(input_assembly_state)
                .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
                .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap()
        };

        let image = StorageImage::general_purpose_image_view(
            &allocator,
            queue.clone(),
            window_renderer.swapchain_image_size(),
            Format::R32G32B32A32_SFLOAT,
            ImageUsage {
                sampled: true,
                storage: true,
                transfer_dst: true,
                ..ImageUsage::empty()
            },
        )
            .unwrap();

        Self {
            queue,
            command_buffer_allocator: StandardCommandBufferAllocator::new(
                allocator.device().clone(),
                Default::default(),
            ),
            descriptor_set_allocator: StandardDescriptorSetAllocator::new(
                allocator.device().clone(),
            ),
            graphics_pipeline,
            render_pass,
            allocator,
            image,
        }
    }
    fn create_render_pass(device: Arc<Device>) -> Arc<RenderPass> {
        let attachment_description = AttachmentDescription {
            format: Some(Format::B8G8R8A8_SRGB),
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
    pub fn draw(
        &mut self,
        before_future: Result<Box<dyn GpuFuture>, AcquireError>,
        target: SwapchainImageView,
        viewport: Viewport,
        pixels: Vec<[[f32; 4]; 2]>,
    ) -> Box<dyn GpuFuture> {
        let frame_buffer = Framebuffer::new(self.render_pass.clone(), FramebufferCreateInfo {
            attachments: vec![target],
            ..Default::default()
        })
            .unwrap();

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
            .unwrap();

        command_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0; 4].into())],
                    ..RenderPassBeginInfo::framebuffer(frame_buffer.clone())
                },
                SubpassContents::SecondaryCommandBuffers,
            )
            .unwrap();

        let command_buffer = self.create_command_buffer(frame_buffer, viewport, pixels);

        let after_future = before_future
            .unwrap()
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap();

        after_future.boxed()
    }
    fn create_command_buffer(&self, frame_buffer: Arc<Framebuffer>, viewport: Viewport, pixels: Vec<[[f32; 4]; 2]>) -> PrimaryAutoCommandBuffer {
        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(self.queue.device().clone(), Default::default());
        let mut builder = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
            .unwrap();

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            &self.allocator,
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            pixels,
        ).expect("Failed to create vertex buffer.");

        builder
            // Before we can draw, we have to *enter a render pass*.
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        frame_buffer.clone(),
                    )
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .set_viewport(0, [viewport.clone()])
            .bind_pipeline_graphics(self.graphics_pipeline.clone())
            .bind_vertex_buffers(0, vertex_buffer.clone())
            .draw(vertex_buffer.len() as u32, 1, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();

        builder.build().unwrap()
    }
}

#[derive(Resource)]
pub struct ViewportResource {
    pub viewport: Arc<Viewport>,
}

impl ViewportResource {
    pub fn get_viewport(&self) -> Viewport {
        Viewport {
            origin: self.viewport.origin,
            dimensions: self.viewport.dimensions,
            depth_range: self.viewport.depth_range.clone(),
        }
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "./src/shaders/shader.vert"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "./src/shaders/shader.frag"
    }
}