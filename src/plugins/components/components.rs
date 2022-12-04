use bevy::ecs::component::Component;
use bevy::ecs::reflect::ReflectComponent;
use bevy_reflect::{FromReflect, Reflect, std_traits::ReflectDefault};

pub use body_subparts::*;

mod body_subparts;

#[derive(Component, Default, Reflect, FromReflect)]
#[reflect(Component, Default)]
pub struct Body {
    pub spine: Vec<Vertebrae>,
}

impl Body {
    pub(crate) fn get_pixels(&self) -> Vec<[[f32; 4]; 2]> {
        let mut pixels = vec![];
        for vertebrae in &self.spine {
            pixels.push([vertebrae.position, vertebrae.color])
        }
        pixels
    }
}

#[derive(Component, Default, Reflect, FromReflect)]
#[reflect(Component, Default)]
pub struct Name(pub String);

#[derive(Component, Default, Reflect, FromReflect)]
#[reflect(Component, Default)]
pub struct Save;