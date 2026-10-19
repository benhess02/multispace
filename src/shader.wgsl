struct DisplayInfo {
    aspect: f32,
}

@group(0) @binding(0) var<uniform> info: DisplayInfo;

@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var tex_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) uv: vec2<f32>
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>
}

@vertex
fn vs_main(vert: Vertex) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(vert.position, 1.0);
    output.color = vec4<f32>(vert.color, 1.0);
    output.uv = vert.uv;
    return output;
}

@fragment
fn fs_main(vert_out: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, tex_sampler, vert_out.uv) * vert_out.color;
}
