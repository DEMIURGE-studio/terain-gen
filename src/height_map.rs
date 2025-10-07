use crate::{HeightMap, CHUNK_SIZE};

#[derive(Clone, Copy, PartialEq)]
pub enum WorldLevel {
    Water,
    Grass,
    Mountain //(u8),
}

pub fn transform_to_height_map(data: Vec<u32>) -> HeightMap {
    let mut heights: HeightMap = HeightMap([[WorldLevel::Water; CHUNK_SIZE]; CHUNK_SIZE]);
    println!("{:?}", data);

    for i in 0..CHUNK_SIZE*CHUNK_SIZE {

        // Take first 8bits of color and compare it to height levels
        let world_level = match data[i] as u8 {
             ..100 => WorldLevel::Water,
            ..140 => WorldLevel::Grass,
            _ => WorldLevel::Mountain, //(data[i] as u8 - 140),
        };

        heights.0[i / 64][i % 64] = world_level;
    }

    heights
}