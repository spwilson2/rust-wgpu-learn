@group(0) @binding(0) var<uniform> uTime: f32;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) color: vec3f,
}
struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var offset = 0.3 * vec3f(cos(uTime), sin(uTime), 0.0);
    out.position = vec4f(in.position + offset, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
    //return vec4f(0.0, 0.4, 1.0, 1.0);
}