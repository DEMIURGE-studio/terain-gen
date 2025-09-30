use std::borrow::Cow;
use std::cmp::PartialEq;
use bevy::asset::{AssetServer, Assets, Handle, RenderAssetUsages};
use bevy::image::Image;
use bevy::prelude::{default, Commands, Res, ResMut, Resource, World};
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph;
use bevy::render::render_graph::RenderGraphContext;
use bevy::render::render_resource::{BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId, CachedPipelineState, ComputePassDescriptor, ComputePipelineDescriptor, Extent3d, PipelineCache, ShaderStages, ShaderType, StorageTextureAccess, TextureDimension, TextureFormat, TextureUsages, UniformBuffer};
use bevy::render::render_resource::binding_types::{texture_storage_2d, uniform_buffer};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;

const SHADER_ASSET_PATH: &str = "./shaders/my_gen.wgsl";
const SIZE: Extent3d = Extent3d {
    width: 64,
    height: 64,
    depth_or_array_layers: 1,
};

#[derive(Resource, Clone, ExtractResource)]
pub struct NoiseImageOutput {
    perlin_texture: Handle<Image>
}

#[derive(Resource, Clone, ExtractResource, ShaderType)]
pub struct NoiseShaderSettings {
    frequency: f32,
    amplitude: f32,
}

#[derive(Resource)]
pub struct NoisePipeline {
    texture_bind_group_layout: BindGroupLayout,
    pipeline_id: CachedComputePipelineId
}

#[derive(Resource)]
struct NoiseImageBindGroup(BindGroup);

pub fn init_perlin_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let texture_bind_group_layout = render_device.create_bind_group_layout(
        "perlin_noise_layout", // Just for debug
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                texture_storage_2d(TextureFormat::Rgba8Unorm, StorageTextureAccess::WriteOnly),
                uniform_buffer::<NoiseShaderSettings>(false),
            ),
        ),
    );
    let shader = asset_server.load(SHADER_ASSET_PATH);
    let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(Cow::from("perlin_noise_pipeline")),
        layout: vec![texture_bind_group_layout.clone()],
        shader: shader.clone(),
        entry_point: Cow::from("main"), // 0.17 Some(Cow::from("main"))

        // In the 0.17 u can use ..default() instead of this, but it's good for understanding
        push_constant_ranges: vec![],
        shader_defs: vec![],
        zero_initialize_workgroup_memory: false,
    });

    commands.insert_resource(NoisePipeline {
        texture_bind_group_layout,
        pipeline_id
    });
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image_size = SIZE.width * SIZE.height * 4;
    let image_data = vec![0; image_size as usize];

    // Image::new_target_texture(); - in the 0.17 u can use this.
    let mut image = Image::new( // Use this in 0.16.*
        SIZE,
        TextureDimension::D2,
        image_data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::MAIN_WORLD
    );

    // THIS IS THE CRITICAL CHANGE
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST       // Destination for a copy (maybe from another texture)
            | TextureUsages::STORAGE_BINDING  // Writable by the compute shader
            | TextureUsages::TEXTURE_BINDING  // Readable by a regular rendering shader
            | TextureUsages::COPY_SRC;        // ALLOWS THE GPU TO COPY *FROM* THIS TEXTURE

    let perlin_handle = images.add(image);

    // Insert your resource so other systems can find the handle
    commands.insert_resource(NoiseImageOutput {
        perlin_texture: perlin_handle,
    });

    // Don't forget to add the shader settings resource
    commands.insert_resource(NoiseShaderSettings {
        frequency: 0.02,
        amplitude: 1.0,
    });
}

pub fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<NoisePipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    perlin_noise_image: Res<NoiseImageOutput>,
    noise_shader_settings: Res<NoiseShaderSettings>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    let noise_image = gpu_images.get(&perlin_noise_image.perlin_texture).unwrap();

    // Uniform buffer is used here to demonstrate how to set up a uniform in a compute shader
    // Alternatives such as storage buffers or push constants may be more suitable for your use case
    let mut uniform_buffer = UniformBuffer::from(noise_shader_settings.clone()); // no need to clone 0.17 (i think ;) )
    uniform_buffer.write_buffer(&render_device, &queue);

    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &BindGroupEntries::sequential((
                &noise_image.texture_view,
                &uniform_buffer
            ))
    );

    commands.insert_resource(NoiseImageBindGroup(bind_group));
}

#[derive(Resource, Clone, Default, ExtractResource, PartialEq)]
pub enum NoiseGenerationRequest {
    #[default]
    Idle, // Do nothing
    Generate, // A request to run the compute shader
}

pub struct PerlinNoiseNode {
    // The node's internal state
    state: NodeState,
}

// The possible states for our node
enum NodeState {
    Loading, // Waiting for the pipeline to be compiled
    Idle,    // Doing nothing
    Generate, // Will run the shader this frame
}

impl Default for PerlinNoiseNode {
    fn default() -> Self {
        Self {
            state: NodeState::Loading,
        }
    }
}

impl render_graph::Node for PerlinNoiseNode {
    // This runs on the CPU before `run`. It's where we check the signal.
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<NoisePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // Check if the pipeline has been compiled
        match self.state {
            NodeState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.pipeline_id)
                {
                    // Pipeline is ready, move to Idle
                    self.state = NodeState::Idle;
                }
            }
            NodeState::Idle | NodeState::Generate => {
                // Get the signal resource that was extracted from the main world
                let mut request = world.resource_mut::<NoiseGenerationRequest>();

                if *request == NoiseGenerationRequest::Generate {
                    // We received a request! Change our internal state to Generate.
                    self.state = NodeState::Generate;
                    // **CRUCIAL**: Reset the request so we don't run again next frame.
                    *request = NoiseGenerationRequest::Idle;
                } else {
                    // No request, so make sure we're idle.
                    self.state = NodeState::Idle;
                }
            }
        }
    }

    // This runs on the GPU command encoder.
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        // Only do something if our internal state is Generate
        if let NodeState::Generate = self.state {
            println!("Compute Node: Received request, running shader.");

            let bind_group = &world.resource::<NoiseImageBindGroup>().0;
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<NoisePipeline>();

            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, bind_group, &[]);

            let compute_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.pipeline_id)
                .unwrap();

            pass.set_pipeline(compute_pipeline);
            pass.dispatch_workgroups(SIZE.width / 8, SIZE.height / 8, 1);
        }

        Ok(())
    }
}

// // This is our settings struct. It will be a component in the MainWorld.
// #[derive(Component, Clone)]
// pub struct NoiseSettings {
//     pub frequency: f32,
//     pub amplitude: f32,
//     pub octaves: u32,
// }
//
// // This is the version of our settings that will exist in the RenderWorld.
// // Bevy's extraction process will create this for us.
// #[derive(Component, Clone, AsBindGroup)]
// struct GpuNoiseSettings {
//     #[uniform(1)]
//     frequency: f32,
//     #[uniform(1)]
//     amplitude: f32,
//     #[uniform(1)]
//     octaves: u32,
// }
//
// // Tell Bevy how to copy NoiseSettings from the MainWorld to the RenderWorld.
// impl ExtractComponent for NoiseSettings {
//     type QueryData = &'static NoiseSettings;
//     type QueryFilter = ();
//     type Out = Self;
//
//     fn extract_component(item: bevy::ecs::query::QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
//         Some(item.clone())
//     }
// }


// use noise::{NoiseFn, Perlin};
//
// pub fn generate_2d_perlin(perlin: Perlin) {
//     const CHUNK_SIZE: usize = 64; // The size of our chunk in one dimension (64x64)
//     const AMPLITUDE: f32 = 10.0;  // How high the mountains can be
//     const FREQUENCY: f64 = 0.05;   // How "zoomed in" the noise is. Higher = more frequent hills.
//
//     let mut positions: Vec<[f32; 3]> = Vec::new();
//     let mut normals: Vec<[f32; 3]> = Vec::new();
//     let mut indices: Vec<u32> = Vec::new();
//
//     // --- 2. GENERATE VERTEX POSITIONS ---
//
//     // We need CHUNK_SIZE + 1 vertices to create CHUNK_SIZE squares.
//     for z in 0..=CHUNK_SIZE {
//         for x in 0..=CHUNK_SIZE {
//             // The input for the noise function. We use f64 for higher precision.
//             let noise_input = [x as f64 * FREQUENCY, z as f64 * FREQUENCY];
//
//             // Get the noise value, which is between -1.0 and 1.0.
//             let noise_value = perlin.get(noise_input);
//
//             // Use the noise value to set the Y-height of the vertex.
//             let y = noise_value as f32 * AMPLITUDE;
//
//             positions.push([x as f32, y, z as f32]);
//             // We'll calculate normals later, so just add a placeholder for now.
//             normals.push([0.0, 1.0, 0.0]);
//         }
//     }
// }