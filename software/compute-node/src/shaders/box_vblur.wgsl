@group(0) @binding(0)
var src_tex: texture_2d<f32>;

@group(0) @binding(1)
var dst_tex: texture_storage_2d<rgba8unorm, write>;

// TODO: Make kernel size configurable
const KERNEL_SIZE: u32 = 10u;
const KERNEL_RADIUS: i32 = i32(KERNEL_SIZE / 2u);

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let size = textureDimensions(src_tex);
    if (gid.x >= size.x || gid.y >= size.y) {
        return;
    }

    var color_sum = vec4<f32>(0.0);
    var weight_sum: f32 = 0.0;

    for (var i = -KERNEL_RADIUS; i <= KERNEL_RADIUS; i++) {
        var y_from = i32(gid.y) + i;

        if (y_from < 0 || y_from >= i32(size.y)) {
            continue;
        }

        let weight = 1.0 / f32(KERNEL_SIZE);
        weight_sum += weight;

        let sampled_color = textureLoad(src_tex, vec2<u32>(gid.x, u32(y_from)), 0);
        
        color_sum += sampled_color * weight;
    }

    textureStore(dst_tex, vec2<u32>(gid.xy), color_sum / weight_sum);
}
