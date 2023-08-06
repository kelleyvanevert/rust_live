// Vertex shader
// ===

struct SystemUniform {
    view_proj: mat4x4<f32>,
};

@group(0)
@binding(0)
var<uniform> system: SystemUniform;

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) center: vec2<f32>,
    @location(2) radius: f32,
    @location(3) trash: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) center: vec2<f32>,
    @location(1) radius: f32,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = system.view_proj * model.position;

    out.center = model.center;
    out.radius = model.radius;

    return out;
}

// Fragment shader
// ===

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sd = distance(in.center, in.position.xy) - in.radius;

    let x = smoothstep(4.0, 5.0, abs(sd));

    return vec4<f32>(0.0, 0.0, 0.0, 1.0 - x);
}
