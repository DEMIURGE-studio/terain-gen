use bevy::asset::Assets;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Commands, EventReader, Mesh, ResMut};
use crate::{ChunkMap, LoadChunkEvent, UnloadChunkEvent};

pub fn spawn_chunk_on_event(
    mut commands: Commands,
    mut reader: EventReader<LoadChunkEvent>,
    mut chunk_map: ResMut<ChunkMap>,
    // You'll need these to create the mesh
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in reader.read() {
        let chunk_pos = event.position;

        if chunk_map.chunks.contains_key(&chunk_pos) {
            continue;
        }

        println!("Loading chunk at: {:?}", chunk_pos);


    }
}

pub fn despawn_chunk_on_event(
    mut commands: Commands,
    mut reader: EventReader<UnloadChunkEvent>,
    mut chunk_map: ResMut<ChunkMap>,
) {
    for event in reader.read() {
        let chunk_pos = event.position;

        // Find the chunk in the map, despawn its entity, and remove it from the map
        if let Some(chunk_data) = chunk_map.chunks.remove(&chunk_pos) {
            println!("Unloading chunk at: {:?}", chunk_pos);
            commands.entity(chunk_data.entity).despawn();
        }
    }
}