mod chunk_manager;
mod perlin;

use std::collections::HashMap;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::{Render, RenderApp, RenderStartup};
use bevy::render::render_graph::{RenderGraph, RenderLabel};
use noise::Perlin;
use crate::chunk_manager::{despawn_chunk_on_event, spawn_chunk_on_event};
use crate::perlin::{ init_perlin_pipeline, prepare_bind_group, setup, NoiseGenerationRequest, NoiseImageOutput, NoiseShaderSettings, PerlinNoiseNode};

// A prelude module is a common pattern in Rust.
// It re-exports all the public types a user of this crate will need.
pub mod prelude {
    // Re-export everything from the parent module that is public.
    pub use super::{
        LoadChunkEvent,
        UnloadChunkEvent,
    };
}

// A component to "tag" an entity as a chunk and store its position.
// This lets you find a chunk in the world and know which one it is.
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct ChunkComponent {
    pub position: IVec2, // Use IVec2 for integer grid coordinates
}

// A struct to hold the logical data for a single chunk.
// This is NOT a component. It will be stored in the manager resource.
#[derive(Debug, Clone)]
pub struct ChunkData {
    // Example: A vector of tile types for this chunk
    // pub tiles: Vec<TileType>,
    pub entity: Entity, // The entity that represents this chunk in the world
}

// The main manager resource. This is the "brain" of your world.
// It keeps track of all loaded chunks.
#[derive(Resource, Default, Debug)]
pub struct ChunkMap {
    // A HashMap is perfect for sparse worlds. It maps a grid coordinate
    // to the corresponding chunk's data.
    pub chunks: HashMap<IVec2, ChunkData>,
    pub chunk_size: i32
}

// An event to request that a specific chunk be loaded.
#[derive(Event, Debug, Message)]
pub struct LoadChunkEvent {
    pub position: IVec2,
}

// An event to request that a chunk be unloaded.
#[derive(Event, Debug, Message)]
pub struct UnloadChunkEvent {
    pub position: IVec2,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct NoiseLabel;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // 1. REGISTER THE TYPES
        app
            // Register the component for reflection (good practice)
            .insert_resource(NoiseGenerationRequest::Idle)
            .register_type::<ChunkComponent>()
            // .insert_resource()
            // Add the event channels
            .add_message::<LoadChunkEvent>()
            .add_message::<UnloadChunkEvent>()
            .insert_resource(ChunkMap {
                chunks: HashMap::new(),
                chunk_size: 64
            });

        app.add_systems(Startup, setup);

        app.add_systems(Update, (
            spawn_chunk_on_event,
            despawn_chunk_on_event
        ));

        // 1. Bridge the worlds: This plugin automatically copies our resources to the Render World.
        app.add_plugins((
            ExtractResourcePlugin::<NoiseGenerationRequest>::default(),
            ExtractResourcePlugin::<NoiseImageOutput>::default(),
            ExtractResourcePlugin::<NoiseShaderSettings>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            // .add_systems(
            //     Render,
            //     ensure_perlin_pipeline_is_initialized.in_set(bevy::render::RenderSet::Prepare),
            // )
            .add_systems(RenderStartup, init_perlin_pipeline)

            // .add_systems(Startup, init_perlin_pipeline)
            // CRITICAL FIX #2: Add the system that prepares the data for the GPU.
            .add_systems(
                Render,
                prepare_bind_group.in_set(bevy::render::RenderSet::PrepareBindGroups),// .run_if(resource_changed::<NoiseGenerationRequest>())
            );


        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(NoiseLabel, PerlinNoiseNode::default());
        render_graph.add_node_edge(NoiseLabel, bevy::render::graph::CameraDriverLabel);
    }
}
