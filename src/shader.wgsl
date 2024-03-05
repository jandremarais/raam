struct VertexInput {
	@location(0) position: vec2<f32>,
	// @builtin(instance_index) instance: u32,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
};

struct Locals {
    screen_size: vec2<f32>,
	offset: vec2<f32>,
    // Uniform buffers need to be at least 16 bytes in WebGL.
    // See https://github.com/gfx-rs/wgpu/issues/2072
    _padding: vec2<u32>,
};
@group(0) @binding(0) var<uniform> r_locals: Locals;

fn position_from_screen(screen_pos: vec2<f32>) -> vec4<f32> {
    return vec4<f32>(
        2.0 * (screen_pos.x + r_locals.offset.x )  / r_locals.screen_size.x - 1.0,
        1.0 - 2.0 * (screen_pos.y + r_locals.offset.y ) / r_locals.screen_size.y,
        0.0,
        1.0,
    );
}

@vertex
fn vs_main(
	model: VertexInput,
) -> VertexOutput {
	var out: VertexOutput;
	// let i = f32(model.instance);
 //    let cell = vec2f(i % grid_size.grid_size.x, floor(i / grid_size.grid_size.x));

 //    let cellOffset = cell / grid_size.grid_size * 2.0;
 //    let gridPos = (vec2<f32>(model.position.x, model.position.y) + 1.0) / grid_size.grid_size - 1.0 + cellOffset;
	out.clip_position = position_from_screen(model.position);
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4f(0.39676, 0.46778, 0.82279, 1.0);
	// return vec4<f32>(0.3, 0.1, 0.1, 1.0);
}

