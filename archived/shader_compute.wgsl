@group(0) @binding(0) var<uniform> size: vec2<u32>;
@group(0) @binding(1) var<storage, read> field_curr: array<u32>;
@group(0) @binding(2) var<storage, read_write> field_next: array<u32>;

@compute @workgroup_size(1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let y = global_id.x;
    let y1 = y - 1u + size.y * u32(y == 0u);
    let y2 = y + 1u - size.y * u32(y == size.y - 1u);
    for (var x: u32 = 0u; x != size.x; x++) {
        let x1 = x - 1u + size.x * u32(x == 0u);
        let x2 = x + 1u - size.x * u32(x == size.x - 1u);

        let B = field_curr[x + y1 * size.x];
        let A = (B << 1u) | (field_curr[x1 + y1 * size.x] >> 31u);
        let C = (B >> 1u) | (field_curr[x2 + y1 * size.x] << 31u);
        let I = field_curr[x + y * size.x];
        let H = (I << 1u) | (field_curr[x1 + y * size.x] >> 31u);
        let D = (I >> 1u) | (field_curr[x2 + y * size.x] << 31u);
        let F = field_curr[x + y2 * size.x];
        let G = (F << 1u) | (field_curr[x1 + y2 * size.x] >> 31u);
        let E = (F >> 1u) | (field_curr[x2 + y2 * size.x] << 31u);

        let AB0 = A ^ B;
        let AB1 = A & B;
        let CD0 = C ^ D;
        let CD1 = C & D;
        let EF0 = E ^ F;
        let EF1 = E & F;
        let GH0 = G ^ H;
        let GH1 = G & H;

        let AD0 = AB0 ^ CD0;
        let AD1 = AB1 ^ CD1 ^ (AB0 & CD0);
        let AD2 = AB1 & CD1;
        let EH0 = EF0 ^ GH0;
        let EH1 = EF1 ^ GH1 ^ (EF0 & GH0);
        let EH2 = EF1 & GH1;

        let AH0 = AD0 ^ EH0;
        let X = AD0 & EH0;
        let Y = AD1 ^ EH1;
        let AH1 = X ^ Y;
        let AH23 = AD2 | EH2 | (AD1 & EH1) | (X & Y);
        let Z = ~AH23 & AH1;
        let I2 = ~AH0 & Z;
        let I3 = AH0 & Z;

        field_next[x + y * size.x] = (I & I2) | I3;
    }
}
