use std::borrow::Cow;
use std::cmp::PartialEq;
use bevy::asset::{AssetServer, Assets, Handle, RenderAssetUsages};
use bevy::image::Image;
use bevy::log::info;
use bevy::prelude::{default, Commands, FromWorld, Message, Res, ResMut, Resource, Trigger, World};
use bevy::render::extract_resource::ExtractResource;
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph;
use bevy::render::render_graph::RenderGraphContext;
use bevy::render::render_resource::{BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId, CachedPipelineState, ComputePassDescriptor, ComputePipelineDescriptor, Extent3d, PipelineCache, ShaderStages, ShaderType, StorageTextureAccess, TextureDimension, TextureFormat, TextureUsages, UniformBuffer};
use bevy::render::render_resource::binding_types::{texture_storage_2d, uniform_buffer};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;

const SHADER_ASSET_PATH: &str = "shaders/my_gen.wgsl";
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

// THIS IS ONLY FOR 0.16
// pub fn ensure_perlin_pipeline_is_initialized(
//     mut commands: Commands,
//     // We ask for an Option<Res<...>>. If it's `None`, the resource doesn't exist yet.
//     pipeline: Option<Res<NoisePipeline>>,
//     render_device: Res<RenderDevice>,
//     asset_server: Res<AssetServer>,
//     pipeline_cache: Res<PipelineCache>,
// ) {
//     // If the pipeline resource already exists, we do nothing and exit early.
//     if pipeline.is_some() {
//         return;
//     }
//
//     // --- If we get here, the pipeline does NOT exist, so we create it. ---
//     // This is the exact same logic from the original init_perlin_pipeline function.
//     let texture_bind_group_layout = render_device.create_bind_group_layout(
//         "perlin_noise_layout",
//         &BindGroupLayoutEntries::sequential(
//             ShaderStages::COMPUTE,
//             (
//                 texture_storage_2d(TextureFormat::Rgba8Unorm, StorageTextureAccess::WriteOnly),
//                 uniform_buffer::<NoiseShaderSettings>(false),
//             ),
//         ),
//     );
//
//     let shader = asset_server.load(SHADER_ASSET_PATH);
//     let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
//         label: Some(Cow::from("perlin_noise_pipeline")),
//         layout: vec![texture_bind_group_layout.clone()],
//         shader: shader.clone(),
//         entry_point: Cow::from("main"),
//         push_constant_ranges: vec![],
//         shader_defs: vec![],
//         zero_initialize_workgroup_memory: false,
//     });
//
//     // We insert the resource, so on the next frame, the `if pipeline.is_some()` check will pass.
//     commands.insert_resource(NoisePipeline {
//         texture_bind_group_layout,
//         pipeline_id,
//     });
// }


// impl FromWorld for NoisePipeline {
//     fn from_world(world: &mut World) -> Self {
//         // This logic is moved directly from your old init_perlin_pipeline function
//         let render_device = world.resource::<RenderDevice>();
//         let pipeline_cache = world.resource::<PipelineCache>();
//
//         let texture_bind_group_layout = render_device.create_bind_group_layout(
//             "perlin_noise_layout", // Just for debug
//             &BindGroupLayoutEntries::sequential(
//                 ShaderStages::COMPUTE,
//                 (
//                     texture_storage_2d(TextureFormat::Rgba8Unorm, StorageTextureAccess::WriteOnly),
//                     uniform_buffer::<NoiseShaderSettings>(false),
//                 ),
//             ),
//         );
//
//         // NOTE: The AssetServer must be retrieved from the main world
//         // This is a crucial detail when running FromWorld in the RenderApp
//         let asset_server = world.resource::<AssetServer>();
//         let shader = asset_server.load(SHADER_ASSET_PATH);
//
//         let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
//             label: Some(Cow::from("perlin_noise_pipeline")),
//             layout: vec![texture_bind_group_layout.clone()],
//             shader: shader.clone(),
//             entry_point: Cow::from("main"),
//             push_constant_ranges: vec![],
//             shader_defs: vec![],
//             zero_initialize_workgroup_memory: false,
//         });
//
//         NoisePipeline {
//             texture_bind_group_layout,
//             pipeline_id
//         }
//     }
// }


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
        entry_point: Some(Cow::from("main")), // 0.17 Some(Cow::from("main"))

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

pub fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_target_texture(SIZE.width, SIZE.height, TextureFormat::Rgba8Unorm);

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST       // Destination for a copy (maybe from another texture)
            | TextureUsages::STORAGE_BINDING  // Writable by the compute shader
            | TextureUsages::TEXTURE_BINDING  // Readable by a regular rendering shader
            | TextureUsages::COPY_SRC;        // ALLOWS THE GPU TO COPY *FROM* THIS TEXTURE

    let perlin_handle = images.add(image);

    // Insert your resource so other systems can find the handle
    commands.insert_resource(NoiseImageOutput {
        perlin_texture: perlin_handle.clone(),
    });

    // Don't forget to add the shader settings resource
    commands.insert_resource(NoiseShaderSettings {
        frequency: 0.02,
        amplitude: 1.0,
    });

    commands.spawn(Readback::texture(perlin_handle)).observe(
        |trigger: Trigger<ReadbackComplete>| {
            // You probably want to interpret the data as a color rather than a `ShaderType`,
            // but in this case we know the data is a single channel storage texture, so we can
            // interpret it as a `Vec<u32>`
            let data: Vec<u32> = trigger.event().to_shader_type();
            info!("Image {:?}", data);
        },
    );
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

#[derive(Resource, Message, Clone, Default, ExtractResource, PartialEq)]
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
                }  else {
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

        // --- 1. GET THE GPU IMAGE FROM THE HANDLE ---

        // Get the resource that contains our Handle<Image>
        let noise_image_output = world.resource::<NoiseImageOutput>();
        let image_handle = &noise_image_output.perlin_texture;

        // Get the RenderAssets resource, which maps handles to their GPU versions
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();

        // Look up our specific GpuImage using the handle.
        // This can fail if the asset hasn't been prepared by the GPU yet,
        // so we handle it gracefully by returning early. This is normal.
        let Some(gpu_image) = gpu_images.get(image_handle) else {
            println!("Got.");
            return Ok(());
        };

        // Now, `gpu_image` is a `&GpuImage`. We can use its fields like `gpu_image.texture`.

        // --- 2. DISPATCH SHADER (your existing code) ---

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
            drop(pass); // End the compute pass
        }

        Ok(())
    }
}
