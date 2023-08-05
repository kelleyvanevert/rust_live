// Vertex shader
// ===

struct SystemUniform {
    view_proj: mat4x4<f32>,
};

@group(0)
@binding(0)
var<uniform> system: SystemUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct InstanceInput {
    @location(1) offset: vec2<f32>,
    @location(2) radius: f32,
    @location(3) trash: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) offset: vec2<f32>,
    @location(1) radius: f32,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = system.view_proj * vec4<f32>(model.position, 1.0);
    // out.position += vec4<f32>(instance.offset, 0.0, 0.0);
    out.offset = instance.offset;
    out.radius = instance.radius;
    return out;
}

// Fragment shader
// ===

struct VarsUniform {
    time: f32,
    radius: f32,
    center: vec2<f32>,
};

@group(0)
@binding(1)
var<uniform> vars: VarsUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sd = distance(vars.center, (in.position.xy - in.offset)) - in.radius;

    let x = step(4.0, abs(sd));

    return vec4<f32>(x, x, x, 1.0 - x);
}
