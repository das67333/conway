use std::arch::x86_64::*;
use std::cell::RefCell;
use std::rc::Rc;
use xxhash_rust::xxh3::{xxh3_128, Xxh3Builder};

type HashMap = std::collections::HashMap<u128, Rc<QuadTreeNode>, Xxh3Builder>;
type Chunk = u64;

const BASE_SIDE: usize = 128;
const CELLS_IN_CHUNK: usize = std::mem::size_of::<Chunk>() * 8;
const CHUNKS_IN_LEAF: usize = BASE_SIDE * BASE_SIDE / CELLS_IN_CHUNK;

#[derive(Default)]
struct LeafCache {
    step: usize, // zero means the cache is empty
    population_densities: Vec<u8>,
}

struct LeafData {
    cells: Box<[Chunk; CHUNKS_IN_LEAF]>,
    cache: RefCell<LeafCache>,
}

enum NodeData {
    Composite([Rc<QuadTreeNode>; 4]),
    Leaf(LeafData),
}

struct QuadTreeNode {
    hash: u128,
    side_log2: u32,
    population_density: f32,
    data: NodeData,
}

pub struct ConwayFieldHash256 {
    root: Rc<QuadTreeNode>,
    size: usize,
    node_updates: HashMap,
    nodes_all: HashMap,
}

impl ConwayFieldHash256 {
    fn get_leaf_node(data: [Chunk; CHUNKS_IN_LEAF], nodes_all: &mut HashMap) -> Rc<QuadTreeNode> {
        #[target_feature(enable = "popcnt")]
        unsafe fn count_ones(data: &[Chunk; CHUNKS_IN_LEAF]) -> u32 {
            data.iter().map(|x| x.count_ones()).sum::<u32>()
        }

        let hash = xxh3_128(bytemuck::bytes_of(&data));
        if let Some(node) = nodes_all.get(&hash) {
            node.clone()
        } else {
            let result = Rc::new(QuadTreeNode {
                hash,
                side_log2: BASE_SIDE.ilog2(),
                population_density: unsafe { count_ones(&data) } as f32
                    / (BASE_SIDE * BASE_SIDE) as f32,
                data: NodeData::Leaf(LeafData {
                    cells: Box::new(data),
                    cache: RefCell::new(LeafCache::default()),
                }),
            });
            nodes_all.insert(hash, result.clone());
            result
        }
    }

    fn get_composite_node(
        nw: &Rc<QuadTreeNode>,
        ne: &Rc<QuadTreeNode>,
        sw: &Rc<QuadTreeNode>,
        se: &Rc<QuadTreeNode>,
        nodes_all: &mut HashMap,
    ) -> Rc<QuadTreeNode> {
        let hash = xxh3_128(bytemuck::bytes_of(&[nw.hash, ne.hash, sw.hash, se.hash]));
        if let Some(node) = nodes_all.get(&hash) {
            node.clone()
        } else {
            let result = Rc::new(QuadTreeNode {
                hash,
                side_log2: nw.side_log2 + 1,
                population_density: (nw.population_density
                    + ne.population_density
                    + sw.population_density
                    + se.population_density)
                    / 4.,
                data: NodeData::Composite([nw.clone(), ne.clone(), sw.clone(), se.clone()]),
            });
            nodes_all.insert(hash, result.clone());
            result
        }
    }

    fn split_node(node: &QuadTreeNode) -> [Rc<QuadTreeNode>; 4] {
        match &node.data {
            NodeData::Composite(nodes) => nodes.clone(),
            NodeData::Leaf(_) => panic!("Base node cannot be split"),
        }
    }

    unsafe fn shift_left(v: __m256i) -> __m256i {
        _mm256_or_si256(
            _mm256_slli_epi64(v, 1),
            _mm256_and_si256(
                _mm256_permute4x64_epi64::<0b10010011>(_mm256_srli_epi64(v, 63)),
                _mm256_set_epi64x(-1, -1, -1, 0),
            ),
        )
    }

    unsafe fn shift_right(v: __m256i) -> __m256i {
        _mm256_or_si256(
            _mm256_srli_epi64(v, 1),
            _mm256_and_si256(
                _mm256_permute4x64_epi64::<0b00111001>(_mm256_slli_epi64(v, 63)),
                _mm256_set_epi64x(0, -1, -1, -1),
            ),
        )
    }

    #[target_feature(enable = "avx2")]
    unsafe fn update_row(row_prev: __m256i, row_curr: __m256i, row_next: __m256i) -> __m256i {
        let b = row_prev;
        let a = Self::shift_left(b);
        let c = Self::shift_right(b);
        let i = row_curr;
        let h = Self::shift_left(i);
        let d = Self::shift_right(i);
        let f = row_next;
        let g = Self::shift_left(f);
        let e = Self::shift_right(f);

        let ab0 = _mm256_xor_si256(a, b);
        let ab1 = _mm256_and_si256(a, b);
        let cd0 = _mm256_xor_si256(c, d);
        let cd1 = _mm256_and_si256(c, d);

        let ef0 = _mm256_xor_si256(e, f);
        let ef1 = _mm256_and_si256(e, f);
        let gh0 = _mm256_xor_si256(g, h);
        let gh1 = _mm256_and_si256(g, h);

        let ad0 = _mm256_xor_si256(ab0, cd0);
        let ad1 = _mm256_xor_si256(_mm256_xor_si256(ab1, cd1), _mm256_and_si256(ab0, cd0));
        let ad2 = _mm256_and_si256(ab1, cd1);

        let eh0 = _mm256_xor_si256(ef0, gh0);
        let eh1 = _mm256_xor_si256(_mm256_xor_si256(ef1, gh1), _mm256_and_si256(ef0, gh0));
        let eh2 = _mm256_and_si256(ef1, gh1);

        let ah0 = _mm256_xor_si256(ad0, eh0);
        let xx = _mm256_and_si256(ad0, eh0);
        let yy = _mm256_xor_si256(ad1, eh1);
        let ah1 = _mm256_xor_si256(xx, yy);
        let ah23 = _mm256_or_si256(
            _mm256_or_si256(ad2, eh2),
            _mm256_or_si256(_mm256_and_si256(ad1, eh1), _mm256_and_si256(xx, yy)),
        );
        let z = _mm256_andnot_si256(ah23, ah1);
        let i2 = _mm256_andnot_si256(ah0, z);
        let i3 = _mm256_and_si256(ah0, z);
        _mm256_or_si256(_mm256_and_si256(i, i2), i3)
    }

    #[target_feature(enable = "avx2")]
    unsafe fn base_update(
        &mut self,
        v0: &Box<[Chunk; CHUNKS_IN_LEAF]>,
        v1: &Box<[Chunk; CHUNKS_IN_LEAF]>,
        v2: &Box<[Chunk; CHUNKS_IN_LEAF]>,
        v3: &Box<[Chunk; CHUNKS_IN_LEAF]>,
    ) -> Rc<QuadTreeNode> {
        const W: usize = BASE_SIDE / CELLS_IN_CHUNK;
        const H: usize = BASE_SIDE;

        let mut src = vec![0; 4 * W * H];
        for y in 0..H {
            for x in 0..W {
                src[x + y * 2 * W] = v0[x + y * W];
                src[(x + W) + y * 2 * W] = v1[x + y * W];
                src[x + (y + H) * 2 * W] = v2[x + y * W];
                src[(x + W) + (y + H) * 2 * W] = v3[x + y * W];
            }
        }

        let mut dst = vec![0; 4 * W * H];
        for t in 1..=H / 2 {
            for y in t..2 * H - t {
                let row_prev = &src[(y - 1) * 2 * W..y * 2 * W];
                let row_prev = _mm256_loadu_si256(row_prev.as_ptr() as *const __m256i);
                let row_curr = &src[y * 2 * W..(y + 1) * 2 * W];
                let row_curr = _mm256_loadu_si256(row_curr.as_ptr() as *const __m256i);
                let row_next = &src[(y + 1) * 2 * W..(y + 2) * 2 * W];
                let row_next = _mm256_loadu_si256(row_next.as_ptr() as *const __m256i);
                let dst = &mut dst[y * 2 * W..(y + 1) * 2 * W];
                let dst = dst.as_mut_ptr() as *mut __m256i;
                _mm256_storeu_si256(dst, Self::update_row(row_prev, row_curr, row_next));
            }
            std::mem::swap(&mut src, &mut dst);
        }

        let mut result = [0; W * H];
        for y in 0..H {
            for x in 0..W {
                result[x + y * W] = src[(x + W / 2) + (y + H / 2) * 2 * W];
            }
        }
        Self::get_leaf_node(result, &mut self.nodes_all)
    }

    fn update_composite(&mut self, nodes: &[Rc<QuadTreeNode>; 4]) -> Rc<QuadTreeNode> {
        let [nw, ne, sw, se] = nodes;
        let [_, ne0, sw0, se0] = Self::split_node(&nw);
        let [nw1, _, sw1, se1] = Self::split_node(&ne);
        let [nw2, ne2, _, se2] = Self::split_node(&sw);
        let [nw3, ne3, sw3, _] = Self::split_node(&se);

        let u1 = Self::get_composite_node(&ne0, &nw1, &se0, &sw1, &mut self.nodes_all);
        let u3 = Self::get_composite_node(&sw0, &se0, &nw2, &ne2, &mut self.nodes_all);
        let u4 = Self::get_composite_node(&se0, &sw1, &ne2, &nw3, &mut self.nodes_all);
        let u5 = Self::get_composite_node(&sw1, &se1, &nw3, &ne3, &mut self.nodes_all);
        let u7 = Self::get_composite_node(&ne2, &nw3, &se2, &sw3, &mut self.nodes_all);

        let p0 = self.update_node(&nw);
        let p1 = self.update_node(&u1);
        let p2 = self.update_node(&ne);
        let p3 = self.update_node(&u3);
        let p4 = self.update_node(&u4);
        let p5 = self.update_node(&u5);
        let p6 = self.update_node(&sw);
        let p7 = self.update_node(&u7);
        let p8 = self.update_node(&se);

        let w0 = Self::get_composite_node(&p0, &p1, &p3, &p4, &mut self.nodes_all);
        let w1 = Self::get_composite_node(&p1, &p2, &p4, &p5, &mut self.nodes_all);
        let w2 = Self::get_composite_node(&p3, &p4, &p6, &p7, &mut self.nodes_all);
        let w3 = Self::get_composite_node(&p4, &p5, &p7, &p8, &mut self.nodes_all);

        let q0 = self.update_node(&w0);
        let q1 = self.update_node(&w1);
        let q2 = self.update_node(&w2);
        let q3 = self.update_node(&w3);
        Self::get_composite_node(&q0, &q1, &q2, &q3, &mut self.nodes_all)
    }

    fn update_node(&mut self, node: &Rc<QuadTreeNode>) -> Rc<QuadTreeNode> {
        if let Some(x) = self.node_updates.get(&node.hash) {
            return x.clone();
        }
        let result = match &node.data {
            NodeData::Composite(nodes) => {
                let [nw, ne, sw, se] = nodes;
                if let (
                    NodeData::Leaf(v0),
                    NodeData::Leaf(v1),
                    NodeData::Leaf(v2),
                    NodeData::Leaf(v3),
                ) = (&nw.data, &ne.data, &sw.data, &se.data)
                {
                    unsafe { self.base_update(&v0.cells, &v1.cells, &v2.cells, &v3.cells) }
                } else {
                    self.update_composite(nodes)
                }
            }
            NodeData::Leaf(_) => unreachable!(),
        };
        self.node_updates.insert(node.hash, result.clone());
        result
    }

    /// Fills the texture of given resolution with a part of field.
    ///
    /// `viewport_x`, `viewport_y` are reduced to divide by `step`;
    ///
    /// `side` is increased to the next power of two;
    ///
    /// `resolution` is reduced to previous power of two (to fit the texture into `dst`),
    /// doesn't exceed `side`;
    ///
    /// `dst` - buffer of texture.
    pub fn fill_texture(
        &self,
        viewport_x: &mut usize,
        viewport_y: &mut usize,
        side: &mut usize,
        resolution: &mut usize,
        dst: &mut Vec<u8>,
    ) {
        fn inner(
            node: &Rc<QuadTreeNode>,
            x: usize,
            y: usize,
            viewport_x: usize,
            viewport_y: usize,
            resolution: usize,
            viewport_side: usize,
            step: usize,
            dst: &mut Vec<u8>,
        ) {
            let half = 1 << (node.side_log2 - 1);
            let step_log2 = step.ilog2();
            if step_log2 == node.side_log2 {
                let j = (x - viewport_x) >> step_log2;
                let i = (y - viewport_y) >> step_log2;
                dst[j + i * resolution] = (node.population_density * u8::MAX as f32) as u8;
                return;
            }
            match &node.data {
                NodeData::Composite(nodes) => {
                    for i in 0..4 {
                        let x = x + half * (i & 1 != 0) as usize;
                        let y = y + half * (i & 2 != 0) as usize;
                        if x + half > viewport_x
                            && x < viewport_x + viewport_side
                            && y + half > viewport_y
                            && y < viewport_y + viewport_side
                        {
                            inner(
                                &nodes[i],
                                x,
                                y,
                                viewport_x,
                                viewport_y,
                                resolution,
                                viewport_side,
                                step,
                                dst,
                            );
                        }
                    }
                }
                NodeData::Leaf(data) => {
                    let mut cache = data.cache.borrow_mut();
                    let k = BASE_SIDE >> step_log2;
                    if cache.step != step {
                        let mut dens = Vec::with_capacity(k * k);
                        for sy in 0..k {
                            for sx in 0..k {
                                let mut sum = 0;
                                for dy in 0..step {
                                    for dx in 0..step {
                                        let x = (sx * step + dx) & (BASE_SIDE - 1);
                                        let y = (sy * step + dy) & (BASE_SIDE - 1);
                                        let pos = (x + y * BASE_SIDE) / CELLS_IN_CHUNK;
                                        let offset = (x + y * BASE_SIDE) % CELLS_IN_CHUNK;
                                        sum += data.cells[pos] >> offset & 1;
                                    }
                                }
                                dens.push(((u8::MAX as Chunk * sum) >> 2 * step.ilog2()) as u8);
                            }
                        }
                        cache.step = step;
                        cache.population_densities = dens;
                    }

                    for sy in 0..k {
                        for sx in 0..k {
                            let j = sx + (x - viewport_x >> step_log2);
                            let i = sy + (y - viewport_y >> step_log2);
                            dst[j + i * resolution] = cache.population_densities[sx + sy * k];
                        }
                    }
                }
            }
        }

        let step = 1 << (*side / *resolution).max(1).ilog2();
        let com_mul = step.max(BASE_SIDE);
        *side = side.next_multiple_of(com_mul) + com_mul;
        *viewport_x = (*viewport_x + 1).next_multiple_of(com_mul) - com_mul;
        *viewport_y = (*viewport_y + 1).next_multiple_of(com_mul) - com_mul;
        *resolution = *side / step;
        dst.fill(0);
        dst.resize(*resolution * *resolution, 0);
        inner(
            &self.root,
            0,
            0,
            *viewport_x,
            *viewport_y,
            *resolution,
            *side,
            step,
            dst,
        );
    }

    pub fn blank(side_log2: u32) -> Self {
        assert!(is_x86_feature_detected!("avx2"));
        assert!(is_x86_feature_detected!("popcnt"));
        assert!(side_log2 > BASE_SIDE.ilog2());
        let mut nodes_all = HashMap::default();
        let mut node = { Self::get_leaf_node([0; CHUNKS_IN_LEAF], &mut nodes_all) };
        for _ in BASE_SIDE.ilog2()..side_log2 {
            node = Self::get_composite_node(&node, &node, &node, &node, &mut nodes_all);
        }
        Self {
            root: node,
            size: 1 << side_log2,
            node_updates: HashMap::default(),
            nodes_all,
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.size, self.size)
    }

    pub fn get_cell(&self, mut x: usize, mut y: usize) -> bool {
        let mut node = &self.root;
        let mut size = self.size;
        while size >= BASE_SIDE {
            size /= 2;
            let idx: usize = (x >= size) as usize + 2 * (y >= size) as usize;
            match &node.data {
                NodeData::Composite(nodes) => node = &nodes[idx],
                NodeData::Leaf(data) => {
                    let pos = (x + y * BASE_SIDE) / CELLS_IN_CHUNK;
                    let offset = (x + y * BASE_SIDE) % CELLS_IN_CHUNK;
                    return data.cells[pos] >> offset & 1 != 0;
                }
            }
            x -= (x >= size) as usize * size;
            y -= (y >= size) as usize * size;
        }
        unreachable!("Size is smaller than the base size, which is impossible")
    }

    pub fn set_cell(&mut self, x: usize, y: usize, state: bool) {
        fn inner(
            mut x: usize,
            mut y: usize,
            mut size: usize,
            node: &Rc<QuadTreeNode>,
            state: bool,
            nodes_all: &mut HashMap,
        ) -> Rc<QuadTreeNode> {
            size /= 2;
            let idx: usize = (x >= size) as usize + 2 * (y >= size) as usize;
            match &node.data {
                NodeData::Composite(nodes) => {
                    let mut nodes = nodes.clone();
                    x -= (x >= size) as usize * size;
                    y -= (y >= size) as usize * size;
                    nodes[idx] = inner(x, y, size, &nodes[idx], state, nodes_all);
                    let [nw, ne, sw, se] = nodes;
                    ConwayFieldHash256::get_composite_node(&nw, &ne, &sw, &se, nodes_all)
                }
                NodeData::Leaf(data) => {
                    let mut data_new: [u64; CHUNKS_IN_LEAF] = data.cells.as_ref().clone();
                    let pos = (x + y * BASE_SIDE) / CELLS_IN_CHUNK;
                    let mask = 1 << ((x + y * BASE_SIDE) % CELLS_IN_CHUNK);
                    if state {
                        data_new[pos] |= mask;
                    } else {
                        data_new[pos] &= !mask;
                    }
                    ConwayFieldHash256::get_leaf_node(data_new, nodes_all)
                }
            }
        }

        self.root = inner(x, y, self.size, &self.root, state, &mut self.nodes_all);
    }

    pub fn update(&mut self, iters_cnt: usize) {
        let m = self.size / 2;
        assert!(
            iters_cnt % m == 0,
            "iters_cnt (={}) is not divisible by {}",
            iters_cnt,
            m
        );
        for _ in 0..iters_cnt / m {
            // TODO: recursive anyway
            let top = &self.root;
            let p = Self::get_composite_node(top, top, top, top, &mut self.nodes_all);
            let q = self.update_node(&p);
            let [se, sw, ne, nw] = Self::split_node(&q);
            self.root = Self::get_composite_node(&nw, &ne, &sw, &se, &mut self.nodes_all);
        }
    }
}
