struct Transform {
    model: mat4x4<f32>,
    normal: mat4x4<f32>,
    bone_offset: u32,
}

struct VertexInput {
    @location(0) vertex: vec3<f32>,
	@location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) joints: vec4<u32>,
    @location(4) weights: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> light_view_projection: mat4x4<f32>;

@group(3) @binding(0)
var<storage, read> transforms: array<Transform>;
@group(3) @binding(1)
var<storage, read> bones: array<mat4x4<f32>>;

const NO_BONES: u32 = 0xffffffffu;

@vertex
fn main(in: VertexInput, @builtin(instance_index) index: u32) -> @builtin(position) vec4<f32> {
    let transform = transforms[index];

    var position = vec4<f32>(in.vertex, 1.0);

    let bone_offset = transform.bone_offset;
    if bone_offset != NO_BONES {
        let skin_matrix =
            bones[bone_offset + in.joints.x] * in.weights.x +
            bones[bone_offset + in.joints.y] * in.weights.y +
            bones[bone_offset + in.joints.z] * in.weights.z +
            bones[bone_offset + in.joints.w] * in.weights.w;
        position = skin_matrix * position;
    }
    
    return light_view_projection * transforms[index].model * position;
}
