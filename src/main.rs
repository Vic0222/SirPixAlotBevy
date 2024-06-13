use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};


fn main() {
    App::new()
        .add_plugins((DefaultPlugins))
        .add_systems(Startup, setup)
        .run();
}

const DIMENSION: i64 = 150;
const PIXEL_SIZE: f32 = 10.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut  shapes = vec![];
    //Mesh2dHandle(meshes.add(Rectangle::new(10.0, 10.0))),

    for _i in 0..(DIMENSION * DIMENSION)  {
        shapes.push(Mesh2dHandle(meshes.add(Rectangle::new(10.0, 10.0))),)
    }
    let num_shapes = shapes.len();
    for (i, shape) in shapes.into_iter().enumerate() {
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

        let x = i as i64 % DIMENSION;
        let x = (x - (DIMENSION  / 2)) as f32 * PIXEL_SIZE;
        let y = i as i64 / DIMENSION ;
        let y = (y as i64 - (DIMENSION /2)) as f32 * PIXEL_SIZE;
        println!("X: {} , Y: {}",x, y);

        commands.spawn(MaterialMesh2dBundle {
            mesh: shape,
            material: materials.add(color),
            transform: Transform::from_xyz(
                // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
                x,
                y,
                0.0,
            ),
            ..default()
        });
    }

    commands.spawn(
        TextBundle::from_section("Press space to toggle wireframes", TextStyle::default())
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            }),
    );
}
