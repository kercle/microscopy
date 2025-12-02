@group(0) @binding(0)
var src_tex: texture_2d<f32>;

@group(0) @binding(1)
var dst_tex: texture_storage_2d<rgba8unorm, write>;

fn shift_vec2(v: vec2<u32>, offset: vec2<i32>, size: vec2<u32>) -> vec2<u32> {
    let x = clamp(i32(v.x) + offset.x, 0, i32(size.x) - 1);
    let y = clamp(i32(v.y) + offset.y, 0, i32(size.y) - 1);
    return vec2<u32>(u32(x), u32(y));
}

fn sobel(gid: vec2<u32>, size: vec2<u32>) -> f32 {
    // Horizontal kernel:
    // -1 0 1
    // -2 0 2
    // -1 0 1

    // Vertical kernel:
    // -1 -2 -1
    //  0  0  0
    //  1  2  1

    var sx: f32 = 0.0;
    var sy: f32 = 0.0;

    var color = textureLoad(src_tex, shift_vec2(gid, vec2<i32>(-1, -1), size), 0).r;
    sx += (-1.0) * color;
    sy += (-1.0) * color;

    color = textureLoad(src_tex, shift_vec2(gid, vec2<i32>(0, -1), size), 0).r;
    sy += (-2.0) * color;

    color = textureLoad(src_tex, shift_vec2(gid, vec2<i32>(1, -1), size), 0).r;
    sx += (1.0) * color;
    sy += (-1.0) * color;

    color = textureLoad(src_tex, shift_vec2(gid, vec2<i32>(-1, 0), size), 0).r;
    sx += (-2.0) * color;

    color = textureLoad(src_tex, shift_vec2(gid, vec2<i32>(1, 0), size), 0).r;
    sx += (2.0) * color;

    color = textureLoad(src_tex, shift_vec2(gid, vec2<i32>(-1, 1), size), 0).r;
    sx += (-1.0) * color;
    sy += (1.0) * color;

    color = textureLoad(src_tex, shift_vec2(gid, vec2<i32>(0, 1), size), 0).r;
    sy += (2.0) * color;

    color = textureLoad(src_tex, shift_vec2(gid, vec2<i32>(1, 1), size), 0).r;
    sx += (1.0) * color;
    sy += (1.0) * color;

    return sqrt(sx * sx + sy * sy);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let size = textureDimensions(src_tex);
    if (gid.x >= size.x || gid.y >= size.y) {
        return;
    }

    let mag = clamp(sobel(gid.xy, size), 0.0, 1.0);
    textureStore(dst_tex, vec2<u32>(gid.xy), vec4<f32>(mag, mag, mag, 1.0));
}
