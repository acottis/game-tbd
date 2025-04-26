struct Camera {
	view: mat4x4<f32>,
	projection: mat4x4<f32>,
}

struct Material {
    base_colour: vec4<f32>,
	has_texture: u32,
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
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> material: Material;
@group(1) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(2)
var s_diffuse: sampler;


@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.projection * camera.view * vec4<f32>(in.vertex, 1.0);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if material.has_texture == 1 {
        return material.base_colour * textureSample(t_diffuse, s_diffuse, in.uv);
    } else {
        return material.base_colour;
    }
}
