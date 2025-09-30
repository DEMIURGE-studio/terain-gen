// STEP 1: Import the pre-written library function.
// Bevy's asset processor will find the file with the matching import path.
#import "noise::perlin_vec2f"

// STEP 2: DEFINE THE BINDINGS.
// This is where you connect your program to the outside world (your Rust code).
// This part was missing from the library because the library doesn't know
// what you want to do with the noise.
@group(0) @binding(0)
var out_texture: texture_storage_2d<rgba8unorm, write>;

// You could also add a buffer for settings
struct Settings {
    frequency: f32,
    amplitude: f32,
};
@group(0) @binding(1)
var<uniform> settings: Settings;


// STEP 3: CREATE THE MAIN ENTRY POINT.
// This is the runnable part of the shader.
@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let texture_size = vec2<f32>(textureDimensions(out_texture));
    if (global_id.x >= u32(texture_size.x) || global_id.y >= u32(texture_size.y)) {
        return;
    }

    let coord = vec2<f32>(f32(global_id.x), f32(global_id.y));

    // Layer 0: The base layer with large shapes
    let frequency0 = settings.frequency * 1.0;
    let amplitude0 = settings.amplitude * 1.0;
    let noise_value_l0 = noise_perlin_vec2f(coord * frequency0) * amplitude0;

    // Layer 1: Double the frequency (more detail), half the amplitude (less influence)
    let frequency1 = settings.frequency * 2.0;
    let amplitude1 = settings.amplitude * 0.5;
    let noise_value_l1 = noise_perlin_vec2f(coord * frequency1) * amplitude1;

    // Layer 2: Double the frequency again, half the amplitude again
    let frequency2 = settings.frequency * 4.0;
    let amplitude2 = settings.amplitude * 0.25;
    let noise_value_l2 = noise_perlin_vec2f(coord * frequency2) * amplitude2;

    // Combine the layers by simply adding them together.
    let combined_noise = noise_value_l0 + noise_value_l1 + noise_value_l2;

    // We need to normalize the result. The maximum possible amplitude is now
    // (amplitude0 + amplitude1 + amplitude2), which is 1.0 + 0.5 + 0.25 = 1.75
    // So we divide by that to bring it back into a predictable range.
    // (This normalization step is important for consistent results!)
    let max_amplitude = amplitude0 + amplitude1 + amplitude2;
    let normalized_noise = combined_noise / max_amplitude;


    // Now, do something with the result.
    // We normalize it from [-1, 1] to [0, 1] for color.
    let color_val = normalized_noise * 0.5 + 0.5;
    let final_color = vec4<f32>(color_val, color_val, color_val, 1.0);

    textureStore(out_texture, vec2<i32>(global_id.xy), final_color);
}