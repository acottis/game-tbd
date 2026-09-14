struct Transform {
    model: mat4x4<f32>,
    normal: mat4x4<f32>,
}

struct VertexInput {
    @location(0) vertex: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> light_view_projection: mat4x4<f32>;

@group(1) @binding(0)
var<storage, read> transforms: array<Transform>;

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) index: u32) -> @builtin(position) vec4<f32> {
    return light_view_projection
        * transforms[index].model
        * vec4<f32>(in.vertex, 1.0);
}
