const VERTICES: array<vec2<f32>, 6> = array(
    vec2<f32>(-1., -1.),
    vec2<f32>(1., -1.),
    vec2<f32>(1., 1.),
    vec2<f32>(1., 1.),
    vec2<f32>(-1., 1.),
    vec2<f32>(-1., -1.),
);

@group(0) @binding(0) var camera_sampler: sampler;
@group(0) @binding(1) var camera_texture: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let pos = VERTICES[index];
    var output: VertexOutput;
    output.position = vec4<f32>(pos, 0.0, 1.0);
    output.uv = vec2<f32>((1. + pos.x) * 0.5, (1. - pos.y) * 0.5);
    return output;
}

@fragment
fn fs_main(vert_out: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(camera_texture, camera_sampler, vert_out.uv);
}
