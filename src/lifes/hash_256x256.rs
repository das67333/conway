use super::ca_trait::CellularAutomaton;
use std::{arch::x86_64::_mm256_loadu2_m128, rc::Rc};
use xxhash_rust::xxh3::{xxh3_128, Xxh3Builder};

type HashMap = std::collections::HashMap<u128, Rc<QuadTreeNode>, Xxh3Builder>;
type Chunk = u64;

#[derive(Clone)]
enum Data {
    Base(Vec<u64>),
    Composite([Rc<QuadTreeNode>; 4]),
}

#[derive(Clone)]
struct QuadTreeNode {
    hash: u128,
    data: Data,
}

pub struct ConwayFieldHash256 {
    root: Rc<QuadTreeNode>,
    size: usize,
    node_updates: HashMap,
}

impl ConwayFieldHash256 {
    const BASE_SIZE: usize = 128;
    const CELLS_IN_CHUNK: usize = std::mem::size_of::<Chunk>() * 8;

    fn rehash(hashes: &[u128; 4]) -> u128 {
        xxh3_128(bytemuck::cast_slice(hashes))
    }

    fn unite_nodes(
        nw: &Rc<QuadTreeNode>,
        ne: &Rc<QuadTreeNode>,
        sw: &Rc<QuadTreeNode>,
        se: &Rc<QuadTreeNode>,
    ) -> QuadTreeNode {
        QuadTreeNode {
            hash: Self::rehash(&[nw.hash, ne.hash, sw.hash, se.hash]),
            data: Data::Composite([nw.clone(), ne.clone(), sw.clone(), se.clone()]),
        }
    }

    fn split_node(node: &QuadTreeNode) -> [Rc<QuadTreeNode>; 4] {
        match &node.data {
            Data::Base(_) => panic!("Base node cannot be split"),
            Data::Composite(nodes) => nodes.clone(),
        }
    }

    unsafe fn update_row(
        row_prev: &[Chunk],
        row_curr: &[Chunk],
        row_next: &[Chunk],
        dst: &mut [Chunk],
    ) {
        let (w, shift) = (row_prev.len(), Self::CELLS_IN_CHUNK - 1);

        let b = row_prev[0];
        let a = b << 1;
        let c = (b >> 1) | (row_prev[1] << shift);
        let i = row_curr[0];
        let h = i << 1;
        let d = (i >> 1) | (row_curr[1] << shift);
        let f = row_next[0];
        let g = f << 1;
        let e = (f >> 1) | (row_next[1] << shift);
        let (ab0, ab1, cd0, cd1) = (a ^ b, a & b, c ^ d, c & d);
        let (ef0, ef1, gh0, gh1) = (e ^ f, e & f, g ^ h, g & h);
        let (ad0, ad1, ad2) = (ab0 ^ cd0, ab1 ^ cd1 ^ (ab0 & cd0), ab1 & cd1);
        let (eh0, eh1, eh2) = (ef0 ^ gh0, ef1 ^ gh1 ^ (ef0 & gh0), ef1 & gh1);
        let (ah0, xx, yy) = (ad0 ^ eh0, ad0 & eh0, ad1 ^ eh1);
        let (ah1, ah23) = (xx ^ yy, ad2 | eh2 | (ad1 & eh1) | (xx & yy));
        let z = !ah23 & ah1;
        let (i2, i3) = (!ah0 & z, ah0 & z);
        dst[0] = (i & i2) | i3;

        for x in 1..w - 1 {
            let (x, x1, x2) = (x, x - 1, x + 1);

            let b = row_prev[x];
            let a = (b << 1) | (row_prev[x1] >> shift);
            let c = (b >> 1) | (row_prev[x2] << shift);
            let i = row_curr[x];
            let h = (i << 1) | (row_curr[x1] >> shift);
            let d = (i >> 1) | (row_curr[x2] << shift);
            let f = row_next[x];
            let g = (f << 1) | (row_next[x1] >> shift);
            let e = (f >> 1) | (row_next[x2] << shift);
            let (ab0, ab1, cd0, cd1) = (a ^ b, a & b, c ^ d, c & d);
            let (ef0, ef1, gh0, gh1) = (e ^ f, e & f, g ^ h, g & h);
            let (ad0, ad1, ad2) = (ab0 ^ cd0, ab1 ^ cd1 ^ (ab0 & cd0), ab1 & cd1);
            let (eh0, eh1, eh2) = (ef0 ^ gh0, ef1 ^ gh1 ^ (ef0 & gh0), ef1 & gh1);
            let (ah0, xx, yy) = (ad0 ^ eh0, ad0 & eh0, ad1 ^ eh1);
            let (ah1, ah23) = (xx ^ yy, ad2 | eh2 | (ad1 & eh1) | (xx & yy));
            let z = !ah23 & ah1;
            let (i2, i3) = (!ah0 & z, ah0 & z);
            dst[x] = (i & i2) | i3;
        }
        let b = row_prev[w - 1];
        let a = (b << 1) | (row_prev[w - 2] >> shift);
        let c = b >> 1;
        let i = row_curr[w - 1];
        let h = (i << 1) | (row_curr[w - 2] >> shift);
        let d = i >> 1;
        let f = row_next[w - 1];
        let g = (f << 1) | (row_next[w - 2] >> shift);
        let e = f >> 1;
        let (ab0, ab1, cd0, cd1) = (a ^ b, a & b, c ^ d, c & d);
        let (ef0, ef1, gh0, gh1) = (e ^ f, e & f, g ^ h, g & h);
        let (ad0, ad1, ad2) = (ab0 ^ cd0, ab1 ^ cd1 ^ (ab0 & cd0), ab1 & cd1);
        let (eh0, eh1, eh2) = (ef0 ^ gh0, ef1 ^ gh1 ^ (ef0 & gh0), ef1 & gh1);
        let (ah0, xx, yy) = (ad0 ^ eh0, ad0 & eh0, ad1 ^ eh1);
        let (ah1, ah23) = (xx ^ yy, ad2 | eh2 | (ad1 & eh1) | (xx & yy));
        let z = !ah23 & ah1;
        let (i2, i3) = (!ah0 & z, ah0 & z);
        dst[w - 1] = (i & i2) | i3;
    }

    #[target_feature(enable = "avx2")]
    unsafe fn update_base(
        &mut self,
        v0: &Vec<Chunk>,
        v1: &Vec<Chunk>,
        v2: &Vec<Chunk>,
        v3: &Vec<Chunk>,
    ) -> Rc<QuadTreeNode> {
        let (w, h) = (Self::BASE_SIZE / Self::CELLS_IN_CHUNK, Self::BASE_SIZE);

        let mut src = vec![0; 4 * w * h];
        for y in 0..h {
            for x in 0..w {
                src[x + y * 2 * w] = v0[x + y * w];
                src[(x + w) + y * 2 * w] = v1[x + y * w];
                src[x + (y + h) * 2 * w] = v2[x + y * w];
                src[(x + w) + (y + h) * 2 * w] = v3[x + y * w];
            }
        }

        let mut dst = vec![0; 4 * w * h];
        for t in 1..=h / 2 {
            for y in t..2 * h - t {
                let row_prev = &src[(y - 1) * 2 * w..y * 2 * w];
                let row_curr = &src[y * 2 * w..(y + 1) * 2 * w];
                let row_next = &src[(y + 1) * 2 * w..(y + 2) * 2 * w];
                let dst = &mut dst[y * 2 * w..(y + 1) * 2 * w];
                Self::update_row(&row_prev, &row_curr, &row_next, dst);
            }
            std::mem::swap(&mut src, &mut dst);
        }

        let mut result = vec![0; w * h];
        for y in 0..h {
            for x in 0..w {
                result[x + y * w] = src[(x + w / 2) + (y + h / 2) * 2 * w];
            }
        }
        Rc::new(QuadTreeNode {
            hash: xxh3_128(bytemuck::cast_slice(&result)),
            data: Data::Base(result),
        })
        // let w = Self::BASE_SIZE / Self::CELLS_IN_CHUNK;
        // let shift = Self::CELLS_IN_CHUNK - 1;
        // for t in 1..=Self::BASE_SIZE / 2 {
        //     for y in 1 + t..Self::BASE_SIZE - t {
        //     }
        // }
        // todo!()
    }

    fn update_composite(&mut self, nodes: &[Rc<QuadTreeNode>; 4]) -> Rc<QuadTreeNode> {
        let [nw, ne, sw, se] = nodes;
        let [_, ne0, sw0, se0] = Self::split_node(&nw);
        let [nw1, _, sw1, se1] = Self::split_node(&ne);
        let [nw2, ne2, _, se2] = Self::split_node(&sw);
        let [nw3, ne3, sw3, _] = Self::split_node(&se);

        let u1 = Self::unite_nodes(&ne0, &nw1, &se0, &sw1);
        let u3 = Self::unite_nodes(&sw0, &se0, &nw2, &ne2);
        let u4 = Self::unite_nodes(&se0, &sw1, &ne2, &nw3);
        let u5 = Self::unite_nodes(&sw1, &se1, &nw3, &ne3);
        let u7 = Self::unite_nodes(&ne2, &nw3, &se2, &sw3);

        let p0 = self.update_node(&nw);
        let p1 = self.update_node(&u1);
        let p2 = self.update_node(&ne);
        let p3 = self.update_node(&u3);
        let p4 = self.update_node(&u4);
        let p5 = self.update_node(&u5);
        let p6 = self.update_node(&sw);
        let p7 = self.update_node(&u7);
        let p8 = self.update_node(&se);

        let w0 = Self::unite_nodes(&p0, &p1, &p3, &p4);
        let w1 = Self::unite_nodes(&p1, &p2, &p4, &p5);
        let w2 = Self::unite_nodes(&p3, &p4, &p6, &p7);
        let w3 = Self::unite_nodes(&p4, &p5, &p7, &p8);

        let q0 = self.update_node(&w0);
        let q1 = self.update_node(&w1);
        let q2 = self.update_node(&w2);
        let q3 = self.update_node(&w3);
        let result = Rc::new(Self::unite_nodes(&q0, &q1, &q2, &q3));
        result
    }

    fn update_node(&mut self, node: &QuadTreeNode) -> Rc<QuadTreeNode> {
        if let Some(x) = self.node_updates.get(&node.hash) {
            return x.clone();
        }
        let result = match &node.data {
            Data::Base(_) => unreachable!(),
            Data::Composite(nodes) => {
                if let [Data::Base(v0), Data::Base(v1), Data::Base(v2), Data::Base(v3)] =
                    nodes.clone().map(|n| n.data.clone())
                {
                    unsafe { self.update_base(&v0, &v1, &v2, &v3) }
                } else {
                    self.update_composite(nodes)
                }
            }
        };
        self.node_updates.insert(node.hash, result.clone());
        result
    }

    fn update(&mut self) {
        let top = &Rc::new(self.root.clone());
        let p = Self::unite_nodes(top, top, top, top);
        let q = self.update_node(&p);
        let [se, sw, ne, nw] = Self::split_node(&q);
        self.root = Rc::new(Self::unite_nodes(&nw, &ne, &sw, &se));
    }
}

impl CellularAutomaton for ConwayFieldHash256 {
    fn id<'a>() -> &'a str {
        "hash"
    }

    fn blank(width: usize, height: usize) -> Self {
        assert!(is_x86_feature_detected!("avx2"));
        assert_eq!(width, height);
        assert!(width >= 2 * Self::BASE_SIZE && width.is_power_of_two());
        let root = {
            let mut node = {
                let data_vec = vec![0; Self::BASE_SIZE * Self::BASE_SIZE / 64];
                let hash = xxh3_128(bytemuck::cast_slice(&data_vec));
                let data = Data::Base(data_vec);
                Rc::new(QuadTreeNode { hash, data })
            };
            let mut s = Self::BASE_SIZE;
            while s != width {
                let hash = Self::rehash(&[node.hash; 4]);
                let data = Data::Composite([0; 4].map(|_| node.clone()));
                node = Rc::new(QuadTreeNode { hash, data });
                s *= 2;
            }
            node
        };
        Self {
            root,
            size: width,
            node_updates: HashMap::default(),
        }
    }

    fn size(&self) -> (usize, usize) {
        (self.size, self.size)
    }

    fn get_cell(&self, mut x: usize, mut y: usize) -> bool {
        let mut node = &self.root;
        let mut size = self.size;
        while size >= Self::BASE_SIZE {
            size /= 2;
            let idx: usize = (x >= size) as usize + 2 * (y >= size) as usize;
            match &node.data {
                Data::Base(data) => {
                    let pos = (x + y * Self::BASE_SIZE) / Self::CELLS_IN_CHUNK;
                    let offset = (x + y * Self::BASE_SIZE) % Self::CELLS_IN_CHUNK;
                    return data[pos] >> offset & 1 != 0;
                }
                Data::Composite(nodes) => node = &nodes[idx],
            }
            x -= (x >= size) as usize * size;
            y -= (y >= size) as usize * size;
        }
        unreachable!("Size is smaller than the base size, which is impossible")
    }

    fn set_cell(&mut self, x: usize, y: usize, state: bool) {
        fn inner(
            mut x: usize,
            mut y: usize,
            mut size: usize,
            node: &Rc<QuadTreeNode>,
            state: bool,
        ) -> Rc<QuadTreeNode> {
            size /= 2;
            let idx: usize = (x >= size) as usize + 2 * (y >= size) as usize;
            match &node.data {
                Data::Base(data) => {
                    let bs = ConwayFieldHash256::BASE_SIZE;
                    let cc = ConwayFieldHash256::CELLS_IN_CHUNK;
                    let mut data_new = data.clone();
                    let pos = (x + y * bs) / cc;
                    let mask = 1 << ((x + y * bs) % cc);
                    if state {
                        data_new[pos] |= mask;
                    } else {
                        data_new[pos] &= !mask;
                    }
                    // TODO: check by hash !!!
                    Rc::new(QuadTreeNode {
                        hash: xxh3_128(&bytemuck::cast_slice(&data_new)),
                        data: Data::Base(data_new),
                    })
                }
                Data::Composite(nodes) => {
                    // TODO: check by hash !!!
                    let mut nodes = nodes.clone();
                    x -= (x >= size) as usize * size;
                    y -= (y >= size) as usize * size;
                    nodes[idx] = inner(x, y, size, &nodes[idx], state);
                    let [nw, ne, sw, se] = nodes;
                    // TODO: check by hash !!!
                    Rc::new(ConwayFieldHash256::unite_nodes(&nw, &ne, &sw, &se))
                }
            }
        }

        self.root = inner(x, y, self.size, &self.root, state);
    }

    fn update(&mut self, iters_cnt: usize) {
        let m = self.size / 2;
        assert!(
            iters_cnt % m == 0,
            "iters_cnt (={}) is not divisible by {}",
            iters_cnt,
            m
        );
        for _ in 0..iters_cnt / m {
            // TODO: recursive anyway
            self.update();
        }
    }
}
