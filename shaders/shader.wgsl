struct Material {
    base_colour: vec4<f32>,
	metallic: f32,
	roughness: f32,
	has_texture: u32,
}

struct Vertex3D {
    translation: mat4x4<f32>,
}

struct VertexInput {
    @location(0) vertex: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) uv: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> vertex3D: Vertex3D;
@group(1) @binding(1)
var<uniform> material: Material;
@group(1) @binding(2)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(3)
var s_diffuse: sampler;


@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera * vertex3D.translation * vec4<f32>(in.vertex, 1.0);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var colour = material.base_colour;
    if material.has_texture == 1 {
        colour *= textureSample(t_diffuse, s_diffuse, in.uv);
    }

    var roughness_effect = mix(1.2, 0.8, material.roughness);
    var metallic_effect = mix(0.9, 1.1, material.metallic);
    var effect = roughness_effect * metallic_effect;
    colour = vec4<f32>(colour.rgb * effect, colour.a);

    return colour;
}
