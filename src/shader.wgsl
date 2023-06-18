struct ExampleUniform {
    color: vec4f,
    time: f32,
}
@group(0) @binding(0) var<uniform> uExampleUniform: ExampleUniform;

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
    let ratio = 0.5;
    let angle = uExampleUniform.time; // you can multiply it go rotate faster
    let alpha = cos(angle);
    let beta = sin(angle);
    var position = vec3<f32>(
        in.position.x,
        alpha * in.position.y + beta * in.position.z,
        alpha * in.position.z - beta * in.position.y,
    );
    //out.position = vec4<f32>(position.x, position.y * ratio, position.z, 1.0);
    out.position = vec4<f32>(position.x, position.y * ratio, position.z * 0.5 + 0.5, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    // possible use of the color uniform, among many others).
    let color = in.color * uExampleUniform.color.rgb;
    // Gamma-correction
    let corrected_color = pow(color, vec3f(2.2));
    return vec4f(corrected_color, uExampleUniform.color.a);
}