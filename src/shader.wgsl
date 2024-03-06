struct CameraUniform {
	offset: vec2<f32>,
	size: vec2<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
	@location(0) pos: vec2<f32>,
}

struct InstanceInput {
	@location(1) offset: vec2<f32>,
	@location(2) scale: vec2<f32>,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
};

fn position_from_screen(screen_pos: vec2<f32>) -> vec4<f32> {
    return vec4<f32>(
        2.0 * (screen_pos.x + camera.offset.x )/ camera.size.x - 1.0,
        1.0 - 2.0 * (screen_pos.y + camera.offset.y) / camera.size.y,
        0.0,
        1.0,
    );
}

@vertex
fn vs_main(
	model: VertexInput,
	instance: InstanceInput,
) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = position_from_screen(model.pos * instance.scale + instance.offset);
	// out.clip_position = position_from_screen(model.pos + instance.offset);
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4f(0.39676, 0.46778, 0.82279, 1.0);
}

