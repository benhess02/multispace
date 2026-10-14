const VERTICES: array<vec2<f32>, 6> = array(
    vec2<f32>(-1., -1.),
    vec2<f32>(1., -1.),
    vec2<f32>(1., 1.),
    vec2<f32>(1., 1.),
    vec2<f32>(-1., 1.),
    vec2<f32>(-1., -1.),
);

struct DisplayInfo {
    aspect: f32,
    tag_x: f32,
    tag_y: f32,
    tag_x2: f32,
    tag_y2: f32,
}

@group(0) @binding(0) var camera_sampler: sampler;
@group(0) @binding(1) var camera_texture: texture_2d<f32>;
@group(0) @binding(2) var<uniform> info: DisplayInfo;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let pos = VERTICES[index];
    var output: VertexOutput;
    output.position = vec4<f32>(pos, 0.0, 1.0);
    output.uv = vec2<f32>((1. + (pos.x * info.aspect)) * 0.5, (1. - pos.y) * 0.5);
    return output;
}

@fragment
fn fs_main(vert_out: VertexOutput) -> @location(0) vec4<f32> {
    if abs(vert_out.uv.x - info.tag_x) < 0.007 && abs(vert_out.uv.y - info.tag_y) < 0.01 {
        return vec4(1., 0., 0., 1.);
    }
    if abs(vert_out.uv.x - info.tag_x2) < 0.007 && abs(vert_out.uv.y - info.tag_y2) < 0.01 {
        return vec4(0., 0., 1., 1.);
    }
    if vert_out.uv.x < 0. || vert_out.uv.x > 1. || vert_out.uv.y < 0. || vert_out.uv.y > 1. {
        return vec4(0., 0., 0., 1.);
    }
    return textureSample(camera_texture, camera_sampler, vert_out.uv);
}
