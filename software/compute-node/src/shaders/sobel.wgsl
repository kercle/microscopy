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

    // Declare as runtime arrays instead of constants
    var gx: array<array<i32, 3>, 3>;
    gx[0] = array<i32, 3>(-1, 0, 1);
    gx[1] = array<i32, 3>(-2, 0, 2);
    gx[2] = array<i32, 3>(-1, 0, 1);

    var gy: array<array<i32, 3>, 3>;
    gy[0] = array<i32, 3>(-1, -2, -1);
    gy[1] = array<i32, 3>(0, 0, 0);
    gy[2] = array<i32, 3>(1, 2, 1);

    var sx: f32 = 0.0;
    var sy: f32 = 0.0;

    for (var j = 0; j < 3; j++) {
        for (var i = 0; i < 3; i++) {
            let x = clamp(i32(gid.x) + i - 1, 0, i32(size.x) - 1);
            let y = clamp(i32(gid.y) + j - 1, 0, i32(size.y) - 1);
            let color = textureLoad(src_tex, vec2<u32>(u32(x), u32(y)), 0).r;

            sx += f32(gx[j][i]) * color;
            sy += f32(gy[j][i]) * color;
        }
    }

    let mag = clamp(sqrt(sx * sx + sy * sy), 0.0, 1.0);
    textureStore(dst_tex, vec2<u32>(gid.xy), vec4<f32>(mag, mag, mag, 1.0));
}
