@group(0) @binding(0)
var src_tex: texture_2d<f32>;

@group(0) @binding(1)
var dst_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let size = textureDimensions(src_tex);
    if (gid.x >= size.x || gid.y >= size.y) {
        return;
    }

    let radius = 13;
    var kernel_weights = array<f32, 27>( 
        0.000229, 0.000606, 0.001446, 0.003246, 0.006737,
        0.013064, 0.024197, 0.040762, 0.063493, 0.090979,
        0.119433, 0.143066, 0.157305, 0.159154, 0.157305,
        0.143066, 0.119433, 0.090979, 0.063493, 0.040762,
        0.024197, 0.013064, 0.006737, 0.003246, 0.001446,
        0.000606, 0.000229
    );

    var color_sum = vec4<f32>(0.0);

    for (var i = -radius; i <= radius; i++) {
        let weight_index = u32(i + radius);
        let weight = kernel_weights[weight_index];

        let x = i32(gid.x);
        let y = clamp(i32(gid.y) + i, 0, i32(size.y) - 1);

        let sampled_color = textureLoad(src_tex, vec2<u32>(u32(x), u32(y)), 0);
        
        color_sum += sampled_color * weight;
    }

    textureStore(dst_tex, vec2<u32>(gid.xy), color_sum);
}