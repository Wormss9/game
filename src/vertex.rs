use vulkano::impl_vertex;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Vertex {
    pub(crate) position: [f32; 4],
}
impl_vertex!(Vertex, position);