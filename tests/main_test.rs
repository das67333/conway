use conway::{life_hash, life_naive, life_shader, life_simd, CellularAutomaton};
use rand::{Rng, SeedableRng};

const N: usize = 128;
const SEED: u64 = 42;
const FILL_RATE: f64 = 0.3;

#[test]
fn test_consistency() {
    let mut life_naive = life_naive::ConwayField::blank(N, N);
    let mut life_simd = life_simd::ConwayField::blank(N, N);
    let mut life_hash = life_hash::ConwayField::blank(N, N);
    let mut life_shader = life_shader::ConwayField::blank(N, N);

    life_naive.randomize(Some(SEED), FILL_RATE);
    life_simd.randomize(Some(SEED), FILL_RATE);
    life_hash.randomize(Some(SEED), FILL_RATE);
    life_shader.randomize(Some(SEED), FILL_RATE);

    life_naive.update(N);
    life_simd.update(N);
    life_hash.update(N);
    life_shader.update(N);

    let v = [
        life_naive.get_cells(),
        life_simd.get_cells(),
        life_hash.get_cells(),
        life_shader.get_cells(),
    ];
    let s = v
        .iter()
        .map(|x| x.iter().map(|x| *x as usize).sum::<usize>())
        .collect::<Vec<_>>();
    assert!(s.iter().all(|x| x == &s[0]), "s={:?}", s);
    assert!(v.iter().all(|x| x == &v[0]));
}

#[test]
fn test_get_single_and_multiple() {
    let mut life_naive = life_naive::ConwayField::blank(N, N);
    let mut life_simd = life_simd::ConwayField::blank(N, N);
    let mut life_hash = life_hash::ConwayField::blank(N, N);
    let mut life_shader = life_shader::ConwayField::blank(N, N);

    life_naive.randomize(Some(SEED), FILL_RATE);
    life_simd.randomize(Some(SEED), FILL_RATE);
    life_hash.randomize(Some(SEED), FILL_RATE);
    life_shader.randomize(Some(SEED), FILL_RATE);

    let cells_naive = life_naive.get_cells();
    let cells_simd = life_simd.get_cells();
    let cells_hash = life_hash.get_cells();
    let cells_shader = life_shader.get_cells();

    let mut iter_naive = cells_naive.iter();
    let mut iter_simd = cells_simd.iter();
    let mut iter_hash = cells_hash.iter();
    let mut iter_shader = cells_shader.iter();
    for y in 0..N {
        for x in 0..N {
            let v = [
                life_naive.get_cell(x, y),
                life_simd.get_cell(x, y),
                life_hash.get_cell(x, y),
                life_shader.get_cell(x, y),
                *iter_naive.next().unwrap(),
                *iter_simd.next().unwrap(),
                *iter_hash.next().unwrap(),
                *iter_shader.next().unwrap(),
            ];
            assert!(v.iter().all(|&x| x == v[0]), "x={} y={} v={:?}", x, y, v);
        }
    }

    let v = [iter_naive.next(), iter_simd.next(), iter_hash.next()];
    assert!(v.iter().all(|&x| x == None), "v={:?}", v);
}

#[test]
fn test_set_single_and_multiple() {
    let mut life_naive_single = life_naive::ConwayField::blank(N, N);
    let mut life_simd_single = life_simd::ConwayField::blank(N, N);
    let mut life_hash_single = life_hash::ConwayField::blank(N, N);
    let mut life_shader_single = life_shader::ConwayField::blank(N, N);

    let mut life_naive_multi = life_naive::ConwayField::blank(N, N);
    let mut life_simd_multi = life_simd::ConwayField::blank(N, N);
    let mut life_hash_multi = life_hash::ConwayField::blank(N, N);
    let mut life_shader_multi = life_shader::ConwayField::blank(N, N);

    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(SEED);
    let states = (0..N * N)
        .map(|_| rng.gen_bool(FILL_RATE))
        .collect::<Vec<_>>();

    life_naive_multi.set_cells(&states);
    life_simd_multi.set_cells(&states);
    life_hash_multi.set_cells(&states);
    life_shader_multi.set_cells(&states);

    for y in 0..N {
        for x in 0..N {
            life_naive_single.set_cell(x, y, states[x + y * N]);
            life_simd_single.set_cell(x, y, states[x + y * N]);
            life_hash_single.set_cell(x, y, states[x + y * N]);
            life_shader_single.set_cell(x, y, states[x + y * N]);
        }
    }

    let v = [
        life_naive_single.get_cells(),
        life_simd_single.get_cells(),
        life_hash_single.get_cells(),
        life_shader_single.get_cells(),
        life_naive_multi.get_cells(),
        life_simd_multi.get_cells(),
        life_hash_multi.get_cells(),
        life_shader_multi.get_cells(),
    ];
    let s = v
        .iter()
        .map(|x| x.iter().map(|x| *x as usize).sum::<usize>())
        .collect::<Vec<_>>();
    assert!(s.iter().all(|x| x == &s[0]), "s={:?}", s);
    assert!(v.iter().all(|x| x == &v[0]));
}
