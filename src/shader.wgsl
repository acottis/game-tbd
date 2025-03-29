struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) colour: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
//    let x = f32(i32(in_vertex_index) - 1);
//    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);

	// vertex_output.position = vec4<f32>(x, y, 0.0, 1.0);
    var positions = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, 1.0),  // Top-left
        vec2<f32>(1.0, 1.0),   // Top-right
        vec2<f32>(1.0, -1.0),  // Bottom-right
        vec2<f32>(-1.0, -1.0)  // Bottom-left
    );

	let position = positions[in_vertex_index] * 0.9;

	var vertex_output: VertexOutput;
    vertex_output.position = vec4<f32>(position, 0., 1.);
    vertex_output.colour = vec4<f32>(0.0, 0.0, 0.0, 1.0);
	return vertex_output;
}

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
	return vertex_output.colour;
}

