struct Material {
    base_colour: vec4<f32>,
	metallic: f32,
	roughness: f32,
	has_texture: u32,
}

struct Light {
	direction: vec3<f32>,
	intensity: f32,
	colour: vec3<f32>,
	ambient: f32,
}

struct VertexInput {
    @location(0) vertex: vec3<f32>,
	@location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) normal: vec3<f32>,
	@location(1) uv: vec2<f32>,
	@location(2) world_position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> light: Light;

@group(2) @binding(0)
var<uniform> material: Material;
@group(2) @binding(1)
var texture: texture_2d<f32>;
@group(2) @binding(2)
var texture_sampler: sampler;

@group(3) @binding(0)
var<storage, read> model_transforms: array<mat4x4<f32>>;

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) index: u32) -> VertexOutput {
    let model_transform = model_transforms[index];
    var out: VertexOutput;
    out.world_position = model_transform * vec4<f32>(in.vertex, 1.0);
    out.position = camera * out.world_position;
    out.uv = in.uv;
    out.normal = normalize((model_transform * vec4<f32>(in.normal, 0.0)).xyz);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var colour = material.base_colour;

    if material.has_texture == 1 {
        colour *= textureSample(texture, texture_sampler, in.uv);
    }

    let light_direction = normalize(-light.direction);

    // How directly the surface faces the light
    let diffuse_strength = max(dot(in.normal, light_direction), 0.0);

    let diffuse = light.colour * diffuse_strength * light.intensity;

    let lighting = light.ambient + diffuse;

    let lit_colour = colour.rgb * lighting;

    return vec4<f32>(lit_colour, colour.a);
}
