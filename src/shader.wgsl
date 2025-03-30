struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) colour: vec4<f32>,
}

@vertex
fn vs_main(
	@location(0) vertex: vec2<f32>, 
	@location(1) texture: vec2<f32>) -> VertexOutput {
	var vertex_output: VertexOutput;
    vertex_output.position = vec4<f32>(vertex, 0.0, 1.0);
    vertex_output.colour = vec4<f32>(texture, 0.0, 1.0);
	return vertex_output;
}

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
	return vertex_output.colour;
}

