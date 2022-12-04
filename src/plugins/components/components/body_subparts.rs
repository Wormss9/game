use bevy::reflect::{FromReflect, Reflect};

#[derive(Default, Reflect, FromReflect)]
pub struct Vertebrae {
    pub position: [f32; 4],
    pub color: [f32; 4],
}