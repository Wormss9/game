use bevy_ecs::system::{Commands, NonSend, NonSendMut, Query, ResMut};
use bevy_vulkano::{BevyVulkanoContext, BevyVulkanoWindows};

use crate::plugins::components::Body;
use crate::plugins::ViewportResource;

use super::resources::VulkanPipeline;

pub fn create_pipelines(mut commands: Commands, context: NonSend<BevyVulkanoContext>, vulkano_windows: NonSend<BevyVulkanoWindows>, viewport: NonSend<ViewportResource>) {
    let primary_window = vulkano_windows.get_primary_window_renderer().unwrap();
    // Create your render pass & pipelines (MyRenderPass could contain your pipelines, e.g. draw_circle)
    let my_pipeline = VulkanPipeline::new(context.context.memory_allocator().clone(), primary_window, viewport.get_viewport());
    // Insert as a resource
    commands.insert_resource(my_pipeline);
}

pub fn render(
    mut vulkano_windows: NonSendMut<BevyVulkanoWindows>,
    mut pipeline: ResMut<VulkanPipeline>,
    viewport: ResMut<ViewportResource>,
    query: Query<&Body>,
) {
    let primary_window = vulkano_windows.get_primary_window_renderer_mut().unwrap();
    let previous_frame_end = primary_window.acquire();
    let target = primary_window.swapchain_image_view();
    let mut pixels:Vec<[[f32; 4]; 2]> = vec![];
    for body in query.iter(){
        pixels.extend(body.get_pixels())
    }

    let future = pipeline.draw(previous_frame_end, target, viewport.get_viewport(),pixels);

    primary_window.present(future, true);
}
