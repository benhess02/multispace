struct DisplayInfo {
    aspect: f32,
}

@group(0) @binding(0) var tex_sampler: sampler;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var<uniform> info: DisplayInfo;

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
    let gamma = vec4(2.2, 2.2, 2.2, 2.2);
    let inv_gamma = 1.0 / gamma;
    return pow(pow(textureSample(tex, tex_sampler, vert_out.uv), inv_gamma) * pow(vert_out.color, inv_gamma), gamma);
}
