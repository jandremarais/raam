struct CameraUniform {
	offset: vec2<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
	@location(0) pos: vec2<f32>,
}

struct InstanceInput {
	@location(1) offset: f32,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
};


@vertex
fn vs_main(
	model: VertexInput,
	instance: InstanceInput,
) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = vec4f(model.pos.x + camera.offset.x, model.pos.y + instance.offset - camera.offset.y, 0.0, 1.0);
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4f(0.39676, 0.46778, 0.82279, 1.0);
}

