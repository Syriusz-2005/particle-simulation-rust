@group(0) @binding(0) var<storage, read> in_positions: array<vec2f>;
@group(0) @binding(1) var<storage, read> in_velocities: array<vec2f>;
@group(0) @binding(2) var<storage, read> in_type_indexes: array<u32>;
@group(0) @binding(3) var<storage, read_write> out_positions: array<vec2f>;
@group(0) @binding(4) var<storage, read_write> out_velocities: array<vec2f>;

@group(0) @binding(5) var<storage, read> in_type_forces: array<f32>;
@group(0) @binding(6) var<storage, read> in_type_radii: array<f32>;
@group(0) @binding(7) var<storage, read> in_type_min_distance: array<f32>;


struct GlobalUniforms {
    screen_size_x: f32,
    screen_size_y: f32,
    particle_types_count: u32,
}
@group(1) @binding(0) var<uniform> global_uniforms: GlobalUniforms;
@group(1) @binding(1) var<storage, read> in_type_masses: array<f32>;
@group(1) @binding(2) var<storage, read> in_type_drag: array<f32>;

const DELTA_T: f32 = 1;

fn get_force(i: u32, j: u32) -> f32 {
    return in_type_forces[i * global_uniforms.particle_types_count + j];
}

fn get_min_distance(i: u32, j: u32) -> f32 {
    return in_type_min_distance[i * global_uniforms.particle_types_count + j];
}

fn get_radii(i: u32, j: u32) -> f32 {
    return in_type_radii[i * global_uniforms.particle_types_count + j];
}

fn remap(x: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    return c + (x - a) * (d - c) / (b - a);
}

@compute
@workgroup_size(64, 1, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>
) {
    let p1_index = global_invocation_id.x;
    let total = arrayLength(&in_positions);
    let screen_size = vec2f(global_uniforms.screen_size_x, global_uniforms.screen_size_y);

    if (p1_index >= total) {
        return;
    }
    
    let p1_pos = in_positions[p1_index];
    let p1_velocity = in_velocities[p1_index];
    let p1_type_index = in_type_indexes[p1_index];
    var total_force = vec2f(0.0);

    for (var p2_index: u32 = 0; p2_index < total; p2_index++) {
        if p1_index == p2_index {
            continue;
        }
        let p2_pos = in_positions[p2_index];
        let p2_velocity = in_velocities[p2_index];
        let p2_type_index = in_type_indexes[p2_index];

        var direction = p2_pos - p1_pos;
        if direction.x > 0.5 * screen_size.x {
            direction.x -= screen_size.x;
        }
        if direction.x < -0.5 * screen_size.x {
            direction.x += screen_size.x;
        }
        if direction.y > 0.5 * screen_size.y {
            direction.y -= screen_size.y;
        }
        if direction.y < -0.5 * screen_size.y {
            direction.y += screen_size.y;
        }
        
        let distance = length(direction);
        direction = normalize(direction);

        let p_min_distance = get_min_distance(p1_type_index, p2_type_index);
        if distance < p_min_distance {
            let force = direction 
                * abs(get_force(p1_type_index, p2_type_index)) 
                * remap(distance, 0.0, p_min_distance, 1.1, 0.0) 
                * -0.204;
            total_force += force;
        }

        let p_radii = get_radii(p1_type_index, p2_type_index);
        if distance < p_radii {
            let force = direction 
                * get_force(p1_type_index, p2_type_index) 
                * remap(distance, 0.0, p_radii, 1.0, 0.0) 
                * 0.084;
            total_force += force;
        }
    }

    let p_mass = in_type_masses[p1_type_index];
    let p_drag = in_type_drag[p1_type_index];
    let p_next_velocity = (p1_velocity + total_force / p_mass) * p_drag;

    let final_position = (p1_pos + p_next_velocity * DELTA_T + screen_size) % screen_size;
    out_positions[p1_index] = final_position;
    out_velocities[p1_index] = p_next_velocity;
}