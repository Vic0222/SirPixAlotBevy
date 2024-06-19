mod components;
use bevy::{
    core_pipeline::bloom::BloomSettings, input::mouse::{MouseButtonInput, MouseMotion}, prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}, utils::HashSet
};
use components::{MousePressed, PixelGrain};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins))
        .add_systems(Startup, (setup_camera))
        .add_systems(Update, (handle_mouse, draw_pixel_grains, despawn_pixel_grains))
        .run();
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
    
    commands.insert_resource(MousePressed(false));
}


// Handle user mouse input for panning the camera around
fn handle_mouse(
    mut button_events: EventReader<MouseButtonInput>,
    mut motion_events: EventReader<MouseMotion>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    mut mouse_pressed: ResMut<MousePressed>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    // Store left-pressed state in the MousePressed resource
    for button_event in button_events.read() {
        if button_event.button != MouseButton::Right {
            continue;
        }
        *mouse_pressed = MousePressed(button_event.state.is_pressed());
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

fn draw_pixel_grains(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    pixel_grains: Query<(Entity, &PixelGrain)>,
) {
    //println!("Entering draw_pixel_grains");
    let (camera, camera_transform) = camera_query.single();
    
    //println!("camera get");
    let Some(view_port_size) = camera.logical_viewport_size() else {
        return;
    };
    let width = view_port_size.x ;
    let height = view_port_size.y;
    let Some(top_left) = camera.viewport_to_world_2d(camera_transform, Vec2::new(0., 0.)) else {
        return;
    };
    let top_left = top_left + Vec2::new(PIXEL_SIZE * -3.0, PIXEL_SIZE * 3.0);
    let top_left = (top_left / 10.0).floor() * 10.0;

    let Some(botton_right) = camera.viewport_to_world_2d(camera_transform, Vec2::new(width, height)) else {
        return;
    };
    
    let botton_right = (botton_right / 10.0).floor() * 10.0;

    let pixel_grains_hash: bevy::utils::hashbrown::HashSet<(i64, i64)> = HashSet::from_iter(pixel_grains.iter().map(|(_, pg)|  (pg.x , pg.y)));
    
    let mut new_pixel_grains = vec![];
    
    for x in ((top_left.x.to_pixel_size() as i64)..(botton_right.x.to_pixel_size() as i64)).step_by(PIXEL_SIZE as usize) {
        //println!("inside loop 1, {}", x);
        for y in ((botton_right.y.to_pixel_size() as i64)..(top_left.y.to_pixel_size() as i64)).step_by(PIXEL_SIZE as usize)  {
            //println!("inside loop 2, {}, {}", x, y);
            if pixel_grains_hash.contains(&(x, y)) {
                //println!("pixel_grains_hash.contains, {}, {}", x, y);
                continue;
            }

            let color = Color::rgb(1.0,0.0,0.0);
            let shape = Mesh2dHandle(meshes.add(Rectangle::new(PIXEL_SIZE, PIXEL_SIZE)));

            //add to a vec so we can batch spawn them.
            new_pixel_grains.push((PixelGrain::new(x as i64, y as i64), MaterialMesh2dBundle {
                mesh: shape,
                material: materials.add(color),
                transform: Transform::from_xyz(
                    // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
                    x.to_pixel_size(), y.to_pixel_size(), 0.0,
                ),
                ..default()
            }))

        }
    }

    commands.spawn_batch(new_pixel_grains);

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
    let width = view_port_size.x ;
    let height = view_port_size.y;
    let Some(top_left) = camera.viewport_to_world_2d(camera_transform, Vec2::new(0., 0.)) else {
        return;
    };
    let top_left = top_left + Vec2::new(PIXEL_SIZE * -3.0, PIXEL_SIZE * 3.0);
    let top_left = (top_left / 10.0).floor() * 10.0;

    let Some(botton_right) = camera.viewport_to_world_2d(camera_transform, Vec2::new(width, height)) else {
        return;
    };
    
    let botton_right = (botton_right / 10.0).floor() * 10.0;


    let world_rect: Rect = Rect::from_corners(top_left, botton_right);
    for (entity, pixel_grain) in pixel_grains.iter() {
        if !world_rect.contains(Vec2::new(pixel_grain.x as f32, pixel_grain.y as f32)) {
            // println!("Despawning pixel_grain, {}, {}", pixel_grain.x, pixel_grain.y);
            // println!("Despawning pixel_grain, {:?}", world_rect);
            // println!("Despawning pixel_grain, {:?}", top_left);
            // println!("Despawning pixel_grain botton_right, {:?}", botton_right);
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