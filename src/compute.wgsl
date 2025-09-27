@group(0) @binding(0) var<storage, read> in_positions: array<vec2f>;
@group(0) @binding(1) var<storage, read> in_velocities: array<vec2f>;
@group(0) @binding(2) var<storage, read> in_type_indexes: array<u32>;
@group(0) @binding(3) var<storage, read_write> out_positions: array<vec2f>;
@group(0) @binding(4) var<storage, read_write> out_velocities: array<vec2f>;

@compute
@workgroup_size(64, 1, 1)
fn main(
  @builtin(global_invocation_id) global_invocation_id: vec3<u32>
) {
  let index = global_invocation_id.x;
  let total = arrayLength(&in_positions);

  if (index >= total) {
    return;
  }

  _ = in_velocities[0];
  _ = in_type_indexes[0];
  _ = out_velocities[0];
  _ = in_positions[0];

  out_positions[global_invocation_id.x] = in_positions[global_invocation_id.x] + vec2f(0.1);
}