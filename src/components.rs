use bevy::prelude::{Component, Resource};

#[derive(Resource)]
pub struct MousePressed(pub bool);


#[derive(Component)]
pub struct PixelGrain {
    pub x: i64,
    pub y: i64,
}

impl PixelGrain {
    
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}