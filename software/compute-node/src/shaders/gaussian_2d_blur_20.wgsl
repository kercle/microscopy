@group(0) @binding(0)
var src_tex: texture_2d<f32>;

@group(0) @binding(1)
var dst_tex: texture_storage_2d<rgba8unorm, write>;

const KERNEL_SIZE: u32 = 60u;
const KERNEL_RADIUS: i32 = i32(KERNEL_SIZE / 2u);

fn gaussian_2d_weight(x: f32, y: f32, sigma: f32) -> f32 {
    let sigma_inv: f32 = 1.0 / sigma;
    let x_div_sigma: f32 = x * sigma_inv;
    let y_div_sigma: f32 = y * sigma_inv;
    let norm_constant: f32 = 0.398942280401;

    let norm_factor = norm_constant * sigma_inv * sigma_inv;
    let exponent = -0.5 * (x_div_sigma * x_div_sigma + y_div_sigma * y_div_sigma);

    return norm_factor * exp(exponent);
}

@compute @workgroup_size(16, 16)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let gid: vec2<u32> = global_id.xy;
    let size = textureDimensions(src_tex);

    if (gid.x >= size.x || gid.y >= size.y) {
        return;
    }

    var result_magnitude_sum: f32 = 0.0;

    for (var j = 0; j < i32(KERNEL_SIZE); j++) {
        for (var i = 0; i < i32(KERNEL_SIZE); i++) {
            let y_offset: i32 = j - KERNEL_RADIUS;
            let x_offset: i32 = i - KERNEL_RADIUS;

            let sample_x: i32 = clamp(i32(gid.x) + x_offset, 0, i32(size.x) - 1);
            let sample_y: i32 = clamp(i32(gid.y) + y_offset, 0, i32(size.y) - 1);

            let magnitude_color = textureLoad(src_tex, vec2<i32>(sample_x, sample_y), 0);
            let edge_magnitude: f32 = magnitude_color.r;

            let weight: f32 = gaussian_2d_weight(f32(x_offset), f32(y_offset), f32(KERNEL_SIZE) / 6.0);

            result_magnitude_sum += edge_magnitude * weight;
        }
    }

    let res: vec4<f32> = vec4<f32>(
        result_magnitude_sum,
        result_magnitude_sum,
        result_magnitude_sum,
        1.0 
    );

    textureStore(dst_tex, vec2<u32>(gid.xy), res);
}
