struct VertexInput {
    @location(0) vertex: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> light_view_projection: mat4x4<f32>;

@group(1) @binding(0)
var<storage, read> model_transforms: array<mat4x4<f32>>;

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) index: u32) -> @builtin(position) vec4<f32> {
    let model = model_transforms[index];

    return light_view_projection
        * model
        * vec4<f32>(in.vertex, 1.0);
}
