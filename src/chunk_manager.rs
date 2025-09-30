use bevy::asset::Assets;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Commands, EventReader, EventWriter, Mesh, MessageReader, ResMut};
use crate::{ChunkMap, LoadChunkEvent, UnloadChunkEvent};
use crate::perlin::{NoiseGenerationRequest, NoiseShaderSettings};

pub fn spawn_chunk_on_event(
    mut commands: Commands,
    mut reader: MessageReader<LoadChunkEvent>,
    mut chunk_map: ResMut<ChunkMap>,
    mut noise_generation_request: ResMut<NoiseGenerationRequest>,
    // We get the writer to send a request to the render world
    // mut noise_request_writer: EventWriter<NoiseGenerationRequest>,
    // We get the settings to update them with the chunk's offset
    mut noise_settings: ResMut<NoiseShaderSettings>,
) {
    for event in reader.read() {
        let chunk_pos = event.position;

        // Don't spawn a chunk if it's already loaded or pending.
        if chunk_map.chunks.contains_key(&chunk_pos) {
            continue;
        }

        println!("Requesting GPU generation for chunk at: {:?}", chunk_pos);

        *noise_generation_request = NoiseGenerationRequest::Generate;
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