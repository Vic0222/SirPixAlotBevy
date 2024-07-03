mod components;
mod dtos;
mod http;

use std::time::Duration;

use bevy::{
    core_pipeline::bloom::BloomSettings, input::mouse::{MouseButtonInput, MouseMotion}, prelude::*, time::common_conditions::on_timer, window::PrimaryWindow
};
use bevy_egui::{egui::{self, color_picker::{color_edit_button_rgb, color_edit_button_srgb}}, EguiContexts, EguiPlugin};
use components::{MouseRightButtonPressed, PickedColor, PixelGrain, PixelRectRequestStatus, PixelRectangle, StatusStreamReceiver, StatusStreamSender, StreamReceiver, StreamSender};
use crossbeam_channel::unbounded;
use dtos::PixelGrainDto;

const API_BASE_URL: &str = "http://172.104.37.82";

fn main() {
    let mut binding = App::new();
    let app = binding
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, (setup_camera))
        .add_systems(Update, (color_picker_system, update_window_size,handle_mouse, draw_gizmos, (spawn_draw_pixel_grains_task.run_if(on_timer(Duration::from_secs(1),)), read_status_stream).chain(), (read_stream, despawn_pixel_grains).chain()));
        
    app.run();
}

fn color_picker_system(mut contexts: EguiContexts, mut picked_color: ResMut<PickedColor>) {
    egui::Window::new("Color Picker").show(contexts.ctx_mut(), |ui| {
        color_edit_button_srgb(ui, &mut picked_color.0);
    });
}


#[cfg(not(target_arch = "wasm32"))]
fn update_window_size() {

}

#[cfg(target_arch = "wasm32")]
fn update_window_size(
    mut windows: Query<&mut Window>,
) {
    let mut window = windows.single_mut();

    let Some(web_window) = web_sys::window() else {
        return;
    };

    let Ok(js_width) = web_window.inner_width() else {
        return;
    };

    let Some(width) = js_width.as_f64() else {
        return;
    };

    let Ok(js_height) = web_window.inner_height() else {
        return;
    };

    let Some(height) = js_height.as_f64() else {
        return;
    };
    let width = width as f32;
    let height = height as f32;

    let current_width = window.width();
    let current_height = window.height();
    if current_width != width && current_height != height {
        window.resolution.set(width, height);
    }
    
}


const PIXEL_SIZE: f32 = 10.0;

/// Camera lerp factor.
const CAM_LERP_FACTOR: f32 = 2.;


fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera::default(),
            ..default()
        },
        BloomSettings::NATURAL,
    ));
    
    commands.insert_resource(MouseRightButtonPressed(false));
    commands.insert_resource(PixelRectRequestStatus::Failed);

    let (tx, rx) = unbounded::<Vec<PixelGrainDto>>();

    commands.insert_resource(StreamReceiver(rx));
    commands.insert_resource(StreamSender(tx));

    let (tx, rx) = unbounded::<PixelRectRequestStatus>();

    commands.insert_resource(StatusStreamReceiver(rx));
    commands.insert_resource(StatusStreamSender(tx));
    commands.insert_resource(PickedColor([255,255,255]));
}


fn draw_gizmos(mut gizmos: Gizmos, q_windows: Query<&Window, With<PrimaryWindow>>, camera_query: Query<(&Camera, &GlobalTransform)>) {
    // Games typically only have one window (the primary window)
    let Some((x, y)) = get_mouse_pixel(&q_windows, &camera_query) else {
        return;
    };

    gizmos.rect_2d(Vec2::new(x, y), 0.0, Vec2::new(PIXEL_SIZE, PIXEL_SIZE), Color::RED);
}

fn get_mouse_pixel(q_windows: &Query<&Window, With<PrimaryWindow>>, camera_query: &Query<(&Camera, &GlobalTransform), ()>) -> Option<(f32, f32)> {
    let Some(position) = q_windows.single().cursor_position() else {
        return None;
    };
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return None;
    };
    
    //println!("view position: {:?}", position);
    let Some(position) = camera.viewport_to_world_2d(camera_transform, position) else {
        return None;
    };
    
    //println!(" world cursor position: {:?}", position);
    let x = (position.x / PIXEL_SIZE).floor() * 10.0;
    let y = (position.y / PIXEL_SIZE).ceil() * 10.0;
    Some((x, y))
}


fn color_to_hex(color: &[u8; 3]) -> String {
    println!("color to hex: {:?}", color);
    format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2])
}

// Handle user mouse input for panning the camera around
fn handle_mouse(
    mut button_events: EventReader<MouseButtonInput>,
    mut motion_events: EventReader<MouseMotion>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    mut mouse_pressed: ResMut<MouseRightButtonPressed>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    sender: Res<StreamSender>,
    picked_color: ResMut<PickedColor>
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    // Store left-pressed state in the MousePressed resource
    for button_event in button_events.read() {
        
        match button_event.button {
            MouseButton::Right => {
                *mouse_pressed = MouseRightButtonPressed(button_event.state.is_pressed());
            },
            MouseButton::Left => {
                if button_event.state.is_pressed() {
                    let url =format!("{}/api/canvas/pixel", API_BASE_URL );
                    let Some((x, y) )= get_mouse_pixel(&q_windows, &camera_query) else {
                        continue;
                    };
                    //get this from mouse position and a color pallete
                    let pixel_dto = PixelGrainDto { x: (x / PIXEL_SIZE) as i64, y: (y / PIXEL_SIZE) as i64, color: color_to_hex(&picked_color.0).to_string()};
                    println!("pixel_dto: {:?}", pixel_dto);
                    let Ok(request): Result<ehttp::Request, serde_json::Error> = http::put_json(url, &pixel_dto) else {
                        continue;
                    };
                    let sender = sender.clone();
                    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
                        
                        println!("Result: {:?}", result);
                        if let Ok(response) = result {
                            if let Ok(dto) = response.json::<PixelGrainDto>(){
                                println!("Dto: {:?}", dto);
                                let _ = sender.send(vec![dto]);
                            }
                        }
                    });
                }
            },
            _ => continue,
        }
        
    }
    
    // If the mouse is not pressed, just ignore motion events
    if !mouse_pressed.0 {
        return;
    }
    
    let (dx, dy) = motion_events
        .read()
        .fold((0.0, 0.0), |(acc_x, acc_y), mouse_motion| (acc_x + mouse_motion.delta.x, acc_y + mouse_motion.delta.y));

    //multiply by -1 to invert becuase we are move the camera itself not the object
    let x = camera.translation.x + (dx * -1.0); 

    //we don't reverse this one as the value is already reversed from the mouse
    let y = camera.translation.y + dy;

    //println!("X: {} , Y: {}, dX {}, dY {}", x, y, dx, dy);
    let direction = Vec3::new(x, y, camera.translation.z);

    camera.translation = camera
        .translation
        .lerp(direction,  CAM_LERP_FACTOR);

       // println!("Box X: {} , Y: {}, Z: {}", camera.translation.x, camera.translation.y, camera.translation.z);
}

fn spawn_draw_pixel_grains_task(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    sender: Res<StreamSender>,
    status_sender: Res<StatusStreamSender>,
    mut rect_request_status: ResMut<PixelRectRequestStatus>
) {
    //println!("Entering draw_pixel_grains");
    let (camera, camera_transform) = camera_query.single();
    
    //println!("camera get");
    let Some(view_port_size) = camera.logical_viewport_size() else {
        return;
    };
    let Some((top_left, botton_right)) = get_rect(view_port_size, camera, camera_transform) else {
        return;
    };

    
    let continue_fetch = match *rect_request_status {
        PixelRectRequestStatus::InProgress => false,
        PixelRectRequestStatus::Failed => true,
        PixelRectRequestStatus::Success(rect) => {
            !(rect.top_left == top_left && rect.botton_right == botton_right)
        },
    };

    //println!("continue_fetch {:?}", &continue_fetch,);

    if !continue_fetch {
        return;
    }

    *rect_request_status = PixelRectRequestStatus::InProgress;
    
    //moved url to a constant
    let url =format!("{}/api/canvas?topLeftX={}&topLeftY={}&bottomRightX={}&bottomRightY={}", API_BASE_URL, top_left.x, top_left.y, botton_right.x, botton_right.y );
    println!("Requesting url {}", url);
    let request = ehttp::Request::get(url);
    let sender = sender.clone();
    let status_sender = status_sender.clone();
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {

        if let Ok(response) = result {
            if let Ok(dto) = response.json::<Vec<PixelGrainDto>>(){
                if let Ok(_) = sender.send(dto) {
                    let _ = status_sender.send(PixelRectRequestStatus::Success(PixelRectangle{top_left, botton_right}));
                    return;
                }
            }
        }
        let _ = status_sender.send(PixelRectRequestStatus::Failed);
        
    });

}

fn get_rect(view_port_size: Vec2, camera: &Camera, camera_transform: &GlobalTransform) -> Option<(Vec2, Vec2)> {
    let width = view_port_size.x ;
    let height = view_port_size.y;
    let Some(top_left) = camera.viewport_to_world_2d(camera_transform, Vec2::new(0., 0.)) else {
        return None;
    };
    
    let top_left = top_left + Vec2::new(PIXEL_SIZE * -3.0, PIXEL_SIZE * 3.0);
    let top_left = (top_left / PIXEL_SIZE).floor() ;

    let Some(botton_right) = camera.viewport_to_world_2d(camera_transform, Vec2::new(width, height)) else {
        return None;
    };
    
    let botton_right = botton_right + Vec2::new(PIXEL_SIZE * 3.0, PIXEL_SIZE * -3.0);
    let botton_right = (botton_right / PIXEL_SIZE).floor();

    
    
    Some((top_left, botton_right))
}

fn read_status_stream(receiver: Res<StatusStreamReceiver>, mut rect_request_status: ResMut<PixelRectRequestStatus>){
    for status in receiver.try_iter() {
        *rect_request_status = status;
    }
}
// This system reads from the receiver and sends events to Bevy
fn read_stream(receiver: Res<StreamReceiver>,
    mut commands: Commands,
    pixel_grains: Query<(Entity, &PixelGrain)>) {

    for pixel_grain_dtos in receiver.try_iter() {
        
        for pixel_grain_dto in pixel_grain_dtos {
            
           let mut entity: Option<Entity> = None;
            for (e, pg) in pixel_grains.iter() {
                if pg.x == pixel_grain_dto.x && pg.y == pixel_grain_dto.y {
                    entity = Some(e);
                    break;
                }
            }
            let entity = match  entity {
                Some(e) => e,
                None => commands.spawn_empty().id(),
            };
            
            let color = Color::hex(pixel_grain_dto.color);
            let color = match color {
                Ok(c) => c,
                Err(_) => Color::GRAY,
            };
            let x = pixel_grain_dto.x as f32 * PIXEL_SIZE;
            let y = pixel_grain_dto.y as f32 * PIXEL_SIZE;

            
            commands.entity(entity).insert(( PixelGrain::new(pixel_grain_dto.x, pixel_grain_dto.y) , 
            SpriteBundle {
                sprite: Sprite {
                    color: color,
                    custom_size: Some(Vec2::new(PIXEL_SIZE, PIXEL_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    x, y, 0.0,
                ),
                ..default()
            }));
            
        }
    
    }
}

fn despawn_pixel_grains(camera_query: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    pixel_grains: Query<(Entity, &PixelGrain)>){

         //println!("Entering draw_pixel_grains");
    let (camera, camera_transform) = camera_query.single();
    
    //println!("camera get");
    let Some(view_port_size) = camera.logical_viewport_size() else {
        return;
    };

    let Some((top_left, botton_right)) = get_rect(view_port_size, camera, camera_transform) else {
        return;
    };

    let world_rect: Rect = Rect::from_corners(top_left, botton_right);
    for (entity, pixel_grain) in pixel_grains.iter() {
        if !world_rect.contains(Vec2::new(pixel_grain.x as f32, pixel_grain.y as f32)) {
            commands.entity(entity).despawn();
        }
    }


}




pub trait ToPixelSize {
    fn to_pixel_size(self) -> f32;
}

impl ToPixelSize for f32 {
    fn to_pixel_size(self) -> f32 {
        (self / PIXEL_SIZE).floor() * PIXEL_SIZE
    }
}

impl ToPixelSize for i64 {
    fn to_pixel_size(self) -> f32 {
        (self as f32 / PIXEL_SIZE).floor() * PIXEL_SIZE
    }
}