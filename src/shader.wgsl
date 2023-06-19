struct ExampleUniform {
    color: vec4f,
    time: f32,
}

@group(0) @binding(0) var<uniform> uExampleUniform: ExampleUniform;
// TODO: Pass in size of the image, so it can repeat.
@group(0) @binding(1) var gradientTexture: texture_2d<f32>;


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
    out.position = vec4f(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    // possible use of the color uniform, among many others).
    let loc = vec2<i32>(in.position.xy);
    let dims = vec2i(textureDimensions(gradientTexture));
    let pos = vec2<i32>(loc.x % dims.x, loc.y % dims.y);
    let color = textureLoad(gradientTexture, pos, 0).rgb;
    // Gamma-correction
    let corrected_color = pow(color, vec3f(2.2));
    //let corrected_color = color;
    return vec4f(corrected_color, uExampleUniform.color.a);
}