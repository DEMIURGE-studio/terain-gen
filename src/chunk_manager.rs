use bevy::prelude::{info, Commands, MessageReader, On, Res, ResMut};
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use crate::{ChunkMap, LoadChunkEvent, UnloadChunkEvent};
use crate::height_map::transform_to_height_map;
use crate::perlin::{NoiseGenerationRequest, NoiseImageOutput};

pub fn spawn_chunk_on_event(
    mut reader: MessageReader<LoadChunkEvent>,
    chunk_map: ResMut<ChunkMap>,
    noise_image_output: Res<NoiseImageOutput>,
    mut commands: Commands,
    mut noise_generation_request: ResMut<NoiseGenerationRequest>
) {
    for event in reader.read() {
        let chunk_pos = event.position;

        // Don't spawn a chunk if it's already loaded or pending.
        if chunk_map.chunks.contains_key(&chunk_pos) {
            continue;
        }

        commands.spawn(Readback::texture(noise_image_output.perlin_texture.clone())).observe(
            |trigger: On<ReadbackComplete>, mut commands: Commands, chunk_map: ResMut<ChunkMap>| {
                commands.entity(trigger.entity).despawn(); // clean event so it will not go in infinite loop

                let data: Vec<u32> = trigger.event().to_shader_type();
                let height_map = transform_to_height_map(data);

                // chunk_map.chunks.insert(chunk_pos, height_map);
            },
        );

        println!("Requesting GPU generation for chunk at: {:?}", chunk_pos);

        *noise_generation_request = NoiseGenerationRequest::Generate;
    }
}


pub fn despawn_chunk_on_event(
    mut commands: Commands,
    mut reader: MessageReader<UnloadChunkEvent>,
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