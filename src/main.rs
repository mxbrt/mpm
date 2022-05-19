use bevy::{
    core_pipeline::node::MAIN_PASS_DEPENDENCIES,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        RenderApp, RenderStage,
    },
    window::WindowDescriptor,
};
use std::borrow::Cow;

use mpm::fluid;

const SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 8;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            // uncomment for unthrottled FPS
            // vsync: false,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_system(fluid_system)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(FluidSimulation {
        image,
        simulator: fluid::Simulator::waterbox(),
    });
}

struct FluidSimulation {
    image: Handle<Image>,
    simulator: fluid::Simulator,
}

fn fluid_system(mut fluid: ResMut<FluidSimulation>, mut image_assets: ResMut<Assets<Image>>) {
    fluid.simulator.step();
    let image = image_assets.get_mut(&fluid.image).unwrap();
    fluid
        .simulator
        .render(&mut image.data, SIZE.0 as usize, SIZE.1 as usize)
}
