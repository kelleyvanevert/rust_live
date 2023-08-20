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
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = system.view_proj * model.position;
    out.color = model.color;
    return out;
}

// Fragment shader
// ===

struct VarsUniform {
    time: f32,
    radius: f32,
    center: vec2<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
