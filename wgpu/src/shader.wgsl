
struct Camera {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>
}

@vertex
fn vertex_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.view_proj * vec4(in.position.xyz, 1.0);
    out.world_position = in.position.xyz;

    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(abs(in.world_position.xyz), 1.0);
}