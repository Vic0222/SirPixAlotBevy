use bevy::{math::Vec2, prelude::{Component, Deref, Resource}};
use bevy_egui::egui::Rect;
use crossbeam_channel::{Receiver, Sender};

use crate::dtos::PixelGrainDto;

#[derive(Resource)]
pub struct MouseRightButtonPressed(pub bool);

#[derive(Resource, Deref)]
pub struct StreamReceiver(pub Receiver<Vec<PixelGrainDto>>);

#[derive(Resource, Deref)]
pub struct StreamSender(pub Sender<Vec<PixelGrainDto>>);

#[derive(Resource, Deref)]
pub struct StatusStreamSender(pub Sender<PixelRectRequestStatus>);

#[derive(Resource, Deref)]
pub struct StatusStreamReceiver(pub Receiver<PixelRectRequestStatus>);

#[derive(Resource, Deref)]
pub struct PickedColor(pub [u8; 3]);

#[derive(Resource, Deref)]
pub struct ColorPickerUIInfoResource(pub ColorPickerUIInfo);


#[derive(Debug)]
pub struct ColorPickerUIInfo{
    pub is_open: bool,
    pub contains_pointer: bool
}


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

#[derive(Resource, Debug)]
pub enum PixelRectRequestStatus{
    InProgress,
    Failed,
    Success(PixelRectangle)
}

#[derive(Clone, Copy, Debug)]
pub struct PixelRectangle {
    pub top_left: Vec2,
    pub botton_right: Vec2,
}