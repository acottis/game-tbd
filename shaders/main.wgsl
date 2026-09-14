struct Transform {
    model: mat4x4<f32>,
    normal: mat4x4<f32>,
}

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
    @location(3) shadow_position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> view_projection: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> light: Light;
@group(1) @binding(1)
var<uniform> light_view_projection: mat4x4<f32>;
@group(1) @binding(2)
var shadow_t: texture_depth_2d;
@group(1) @binding(3)
var shadow_s: sampler_comparison;


@group(2) @binding(0)
var<uniform> material: Material;
@group(2) @binding(1)
var material_t: texture_2d<f32>;
@group(2) @binding(2)
var material_s: sampler;

@group(3) @binding(0)
var<storage, read> transforms: array<Transform>;

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) index: u32) -> VertexOutput {
    let transform = transforms[index];
    let world_position = transform.model * vec4<f32>(in.vertex, 1.0);

    var out: VertexOutput;
    out.position = view_projection * world_position;
    out.uv = in.uv;
    out.normal = (transform.normal * vec4<f32>(in.normal, 0.0)).xyz;
    
    out.shadow_position = light_view_projection * world_position;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    var colour = material.base_colour;

    if material.has_texture == 1 {
        colour *= textureSample(material_t, material_s, in.uv);
    }

    // Lighting
    let light_direction = normalize(-light.direction);
    // How directly the surface faces the light
    let diffuse_strength = max(dot(normal, light_direction), 0.0);
    let diffuse = light.colour * diffuse_strength * light.intensity;

    // Shadows
    let shadow_coordinates = in.shadow_position.xyz / in.shadow_position.w;
    // Convert from [-1, 1] to texture coordinates [0, 1].
    let shadow_uv = vec2<f32>(
        shadow_coordinates.x * 0.5 + 0.5,
        -shadow_coordinates.y * 0.5 + 0.5
    );
    let shadow = textureSampleCompare(
        shadow_t,
        shadow_s,
        shadow_uv,
        shadow_coordinates.z - 0.005
    );

    // Add ambient to prevent lighting being 0
    let lighting = light.ambient + (diffuse * shadow);

    let lit_colour = colour.rgb * lighting;
    return vec4<f32>(lit_colour, colour.a);
}
