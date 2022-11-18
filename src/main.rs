use std::borrow::Borrow;

use winit::event_loop::EventLoop;

use vertex::Vertex;
use vulkan::Vulkan;

mod vulkan;
mod vertex;

fn main() {
    let vertices = [
        Vertex {
            position: [0.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.1, 0.1, 0.1, 1.0],
        },
        Vertex {
            position: [0.2, 0.2, 0.2, 1.0],
        },
        Vertex {
            position: [0.3, 0.3, 0.3, 1.0],
        },
    ];

    let mut test = Test::new();

    let event_loop = EventLoop::new();
    let mut vulkan = Vulkan::new(&event_loop, vertices);

    event_loop.run(move |event, _, control_flow| vulkan.event_handler(event, control_flow, &mut test));
}

#[derive(Clone, Copy)]
pub struct Test {
    pub vertices: [Vertex; 4],
}

impl Test {
    pub fn new() -> Self {
        Self {
            vertices: [
                Vertex {
                    position: [0.0, 0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [0.0, 0.0, 0.1, 1.0],
                },
                Vertex {
                    position: [0.0, 0.0, 0.2, 1.0],
                },
                Vertex {
                    position: [0.0, 0.0, 0.3, 1.0],
                },
            ]
        }
    }
    pub fn get(&mut self) -> [Vertex; 4] {
        for (i, vertex) in self.clone().vertices.iter().enumerate() {
            for (j, _number) in vertex.position.iter().enumerate() {
                if j < 2 {
                    if rand::random::<bool>() {
                        self.vertices[i].position[j] += rand::random::<f32>() / 100.0;
                    } else {
                        self.vertices[i].position[j] -= rand::random::<f32>() / 100.0;
                    }
                    if self.vertices[i].position[j] > 1.0 { self.vertices[i].position[j] = -1.0 };
                    if self.vertices[i].position[j] < -1.0 { self.vertices[i].position[j] = 1.0 };
                }
            }
        }
        self.vertices
    }
}