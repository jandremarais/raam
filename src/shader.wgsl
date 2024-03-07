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
	@location(3) alpha: f32,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) alpha: f32,
};

fn position_from_screen(screen_pos: vec2<f32>) -> vec4<f32> {
    return vec4<f32>(
        2.0 * (screen_pos.x + camera.offset.x )/ camera.size.x - 1.0,
        1.0 - 2.0 * (screen_pos.y + camera.offset.y) / camera.size.y,
        0.0,
        1.0,
    );
}


fn srgb_to_linear(c: f32) -> f32 {
	let cf = c / 255.0;
    if cf <= 0.04045 {
        return cf / 12.92;
    } else {
        return pow((cf + 0.055) / 1.055, 2.4);
    }
}


@vertex
fn vs_main(
	model: VertexInput,
	instance: InstanceInput,
) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = position_from_screen(model.pos * instance.scale + instance.offset);
	out.alpha = srgb_to_linear(instance.alpha);
	// out.alpha = instance.alpha;
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4f(0.39676, 0.46778, 0.82279, in.alpha);
	// return vec4f(0.39676, 0.46778, 0.82279, 0.1);
}

