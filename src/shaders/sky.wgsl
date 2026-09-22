@group(0) @binding(0)
var<uniform> inverse_view_projection: mat4x4<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );

    var out: VertexOutput;

    out.position = vec4<f32>(positions[i], 1.0, 1.0);
    out.ndc = positions[i];

    return out;
}

@fragment
fn fs_main(
    @location(0) ndc: vec2<f32>,
) -> @location(0) vec4<f32> {

    let clip_position = vec4<f32>(ndc.x, ndc.y, 1.0, 1.0);
    let world_position = inverse_view_projection * clip_position;
    let ray_direction = normalize(world_position.xyz / world_position.w);

    let height = ray_direction.y;
    // Make the transition non linear
    // ___
    //    \
    //     \
    //      \___
    let t = smoothstep(-0.05, 0.25, height);

    let horizon_color = vec3<f32>(0.78, 0.84, 0.90);
    let zenith_color = vec3<f32>(0.18, 0.32, 0.55);

    let sky_color = mix(horizon_color, zenith_color, t);

    return vec4<f32>(sky_color, 1.0);
}
