use bevy::prelude::IVec2;
use rand::prelude::SmallRng;
use rand::{RngCore, SeedableRng, TryRngCore};
use crate::height_map::WorldLevel;
use crate::{HeightMap, CHUNK_SIZE};

///Get random value in range of -0.5 and 0.5
#[inline(always)]
fn random_val(rng : &mut SmallRng) -> f32 {
    let random_val = (rng.next_u32() as f32) / (u32::MAX as f32) - 0.5;
    println!("random value: {}", random_val);
    random_val
}

#[derive(Default)]
struct SearchIterator {
    pub current: SearchIteration,
}

#[derive(PartialEq, Default)]
enum SearchIteration {
    #[default]
    Top,
    Right,
    Bottom,
    Left,
    None,
}

impl Iterator for SearchIterator {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == SearchIteration::Top {
            self.current = SearchIteration::Right;
            return Some(IVec2::new(0, 1))
        } else if self.current == SearchIteration::Right {
            self.current = SearchIteration::Bottom;
            return Some(IVec2::new(1, 0))
        } else if self.current == SearchIteration::Bottom {
            self.current = SearchIteration::Left;
            return Some(IVec2::new(0, -1))
        } else if self.current == SearchIteration::Left {
            self.current = SearchIteration::None;
            return Some(IVec2::new(-1, 0))
        }
        self.current = SearchIteration::Top;
        None
    }
}

pub fn plane_to_vertex_border_with_normal(
    height_map: &HeightMap,
    target_level: WorldLevel,
    find_difference_with: WorldLevel
) {
    // let _to_continue: IVec2;
    let mut point_to_continue: IVec2 = IVec2::new(-1, -1); // Default values, we will check by this is anything found

    'outer: for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            if height_map.0[x][y] == target_level {
                if (x > 0 && height_map.0[x-1][y] == find_difference_with) ||
                    (x < CHUNK_SIZE - 1 && height_map.0[x+1][y] == find_difference_with) ||
                    (y > 0 && height_map.0[x][y-1] == find_difference_with) ||
                    (y < CHUNK_SIZE - 1 && height_map.0[x][y + 1] == find_difference_with) {
                    point_to_continue = IVec2::new(x as i32, y as i32);

                    break 'outer;
                }
            }
        }
    }

    if point_to_continue.x == -1 { // -1 only possible if it's a default value
        return;
    }

    let mut rng = SmallRng::seed_from_u64(point_to_continue.x as u64);

    let mut previous_value = point_to_continue;

    let random_vertex_shift =[random_val(&mut rng), random_val(&mut rng)];

    loop {
        let search_iterator = SearchIterator::default();
    }
}