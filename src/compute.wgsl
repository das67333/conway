@group(0) @binding(0) var<uniform> size: vec2<u32>;
@group(0) @binding(1) var<storage, read> field_curr: array<u32>;
@group(0) @binding(2) var<storage, read_write> field_next: array<u32>;

fn get_cell(x: u32, y: u32) -> u32 {
    return (field_curr[(x + y * size.x) >> 5u] >> (x & 31u)) & 1u; // TODO parentheses
}

fn set_cell(x: u32, y: u32, state: bool) {
    let pos = (x + y * size.x) >> 5u;
    let mask = 1u << (x & 31u);
    if state {
        field_next[pos] |= mask;
    } else {
        field_next[pos] &= ~mask;
    }
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) grid: vec3<u32>) {
    let y = grid.x;
    var y1: u32;
    if y == 0u {
        y1 = size.y - 1u;
    } else {
        y1 = y - 1u;
    }
    var y2: u32;
    if y == size.y - 1u {
        y2 = 0u;
    } else {
        y2 = y + 1u;
    }
    for (var x: u32 = 0u; x < size.x; x++) {
        var x1: u32;
        if x == 0u {
            x1 = size.x - 1u;
        } else {
            x1 = x - 1u;
        }
        var x2: u32;
        if x == size.x - 1u {
            x2 = 0u;
        } else {
            x2 = x + 1u;
        }
        let n = get_cell(x1, y1) + get_cell(x, y1) + 
            get_cell(x2, y1) + get_cell(x1, y) + 
            get_cell(x2, y) + get_cell(x1, y2) + 
            get_cell(x, y2) + get_cell(x2, y2);
        if bool(get_cell(x, y)) { // TODO: simplify
            set_cell(x, y, n == 2u || n == 3u);
        } else {
            set_cell(x, y, n == 3u);
        }
    }
}
