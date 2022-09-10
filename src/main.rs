use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, Extent3d, TextureDimension, TextureFormat},
    render::texture::ImageSettings,
    sprite::{MaterialMesh2dBundle, Material2d, Material2dPlugin},
    input::mouse::MouseWheel,
};

fn main() {
    App::new()
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_plugin(Material2dPlugin::<TileMapMaterial>::default())
        .add_startup_system(setup)
        .add_system(camera_control_system)
        .run();
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "1d1819f6-fa54-4940-9c76-fad1024d2e71"]
struct TileMapMaterial {
    #[texture(0)]
    #[sampler(1)]
    tile_map_texture: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    tile_set_texture: Handle<Image>,
}

const TILE_MAP_FRAGMENT_SHADER: &str = r"
// #import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_bindings

@group(1) @binding(0)
var tile_map_texture: texture_2d<f32>;
@group(1) @binding(1)
var tile_map_sampler: sampler;

@group(1) @binding(2)
var tile_set_texture: texture_2d<f32>;
@group(1) @binding(3)
var tile_set_sampler: sampler;

fn sample_tile_map(uv: vec2<f32>) -> vec4<f32> {
    let tile_id = textureSample(tile_map_texture, tile_map_sampler, uv);
    
    let tile_map_size = vec2<f32>(textureDimensions(tile_map_texture));
    let tile_coords = uv * tile_map_size;
    let px_coords_in_tile = tile_coords - floor(tile_coords);
    
    let tile_set_size = vec2<f32>(textureDimensions(tile_set_texture));
    let tile_set_texture_coords = (tile_id.xy * 255.0 + px_coords_in_tile) * 8.0 / tile_set_size;

    let color = textureSample(tile_set_texture, tile_set_sampler, tile_set_texture_coords);
    return color;
}

@fragment
fn fragment(
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    /*
    let off_x = 0.25 / view.width;
    let off_y = 0.25 / view.height;

    let sample_1 = sample_tile_map(uv + vec2(-off_x, -off_y));
    let sample_2 = sample_tile_map(uv + vec2( off_x, -off_y));
    let sample_3 = sample_tile_map(uv + vec2(-off_x,  off_y));
    let sample_4 = sample_tile_map(uv + vec2( off_x,  off_y));

    return (sample_1 + sample_2 + sample_3 + sample_4) / 4.0;
    */
    return sample_tile_map(uv);
}
";

const TILE_MAP_FRAGMENT_SHADER_HANDLE: HandleUntyped = 
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13828845428412094821);

impl Material2d for TileMapMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(TILE_MAP_FRAGMENT_SHADER_HANDLE.typed())
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut materials: ResMut<Assets<TileMapMaterial>>,
    mut images: ResMut<Assets<Image>>,
    assets: Res<AssetServer>,
) {
    shaders.set_untracked(
        TILE_MAP_FRAGMENT_SHADER_HANDLE,
        Shader::from_wgsl(TILE_MAP_FRAGMENT_SHADER)
    );

    let tile_map_texture = {
        let width = 16;
        let height = 16;

        let mut data = Vec::with_capacity((width * height * 2) as usize);

        for y in 0..2 {
            for x in 0..16 {
                data.push(x % 2);
                data.push(y % 2);
            }
        }

        for _y in 2..16 {
            for _x in 0..16 {
                data.push(8);
                data.push(0);
            }
        }

        let mut image = Image::new(Extent3d {
            width, height, depth_or_array_layers: 1,
        }, TextureDimension::D2, data, TextureFormat::Rg8Unorm);
        image.sampler_descriptor = bevy::render::texture::ImageSampler::nearest();

        images.add(image)
    };

    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::default().with_scale(Vec3::splat(128.)),
        material: materials.add(TileMapMaterial {
            tile_map_texture,
            tile_set_texture: assets.load("test-tileset.png"),
        }),
        ..default()
    });
}

fn camera_control_system(
    mut scroll_evr: EventReader<MouseWheel>,
    mut query: Query<(&mut Transform, With<Camera>)>
) {
    use bevy::input::mouse::MouseScrollUnit;
    for ev in scroll_evr.iter() {
        match ev.unit {
            MouseScrollUnit::Line => {
                let dzoom = ev.y;
                for (mut transform, _) in query.iter_mut() {
                    let prev_scale = transform.scale.x;
                    let new_scale = (prev_scale + dzoom / 10.0).min(1.0).max(1.0 / 8.0);
                    transform.scale = Vec3::new(new_scale, new_scale, 1.0);
                }
            },
            _ => { todo!() }
        }
    }
}