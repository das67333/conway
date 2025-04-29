use super::{MemoryManager, NodeIdx, PrefetchedNode, LEAF_SIZE, LEAF_SIZE_LOG2};
use crate::{GoLEngine, Topology, Pattern};
use ahash::AHashMap as HashMap;
use anyhow::Result;
use num_bigint::BigInt;

/// Implementation of [HashLife algorithm](https://conwaylife.com/wiki/HashLife)
pub struct HashLifeEngine<Meta> {
    pub(super) size_log2: u32,
    pub(super) root: NodeIdx,
    pub(super) generations_per_update_log2: u32,
    pub(super) has_cache: bool,
    pub(super) mem: MemoryManager<Meta>,
    pub(super) topology: Topology,
}

impl<Meta: Clone + Default> HashLifeEngine<Meta> {
    fn update_row(row_prev: u16, row_curr: u16, row_next: u16) -> u16 {
        let b = row_prev;
        let a = b << 1;
        let c = b >> 1;
        let i = row_curr;
        let h = i << 1;
        let d = i >> 1;
        let f = row_next;
        let g = f << 1;
        let e = f >> 1;

        let ab0 = a ^ b;
        let ab1 = a & b;
        let cd0 = c ^ d;
        let cd1 = c & d;

        let ef0 = e ^ f;
        let ef1 = e & f;
        let gh0 = g ^ h;
        let gh1 = g & h;

        let ad0 = ab0 ^ cd0;
        let ad1 = (ab1 ^ cd1) ^ (ab0 & cd0);
        let ad2 = ab1 & cd1;

        let eh0 = ef0 ^ gh0;
        let eh1 = (ef1 ^ gh1) ^ (ef0 & gh0);
        let eh2 = ef1 & gh1;

        let ah0 = ad0 ^ eh0;
        let xx = ad0 & eh0;
        let yy = ad1 ^ eh1;
        let ah1 = xx ^ yy;
        let ah23 = (ad2 | eh2) | (ad1 & eh1) | (xx & yy);
        let z = !ah23 & ah1;
        let i2 = !ah0 & z;
        let i3 = ah0 & z;
        (i & i2) | i3
    }

    /// `nw`, `ne`, `sw`, `se` must be leaves
    pub(super) fn update_leaves(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        steps: u64,
    ) -> NodeIdx {
        let [nw, ne, sw, se] =
            [nw, ne, sw, se].map(|x| self.mem.get(x, LEAF_SIZE_LOG2).leaf_cells());

        let mut src: [u16; 16] = nw
            .iter()
            .zip(ne.iter())
            .chain(sw.iter().zip(se.iter()))
            .map(|(&l, &r)| u16::from_le_bytes([l, r]))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut dst = [0; 16];

        for t in 1..=steps as usize {
            for y in t..16 - t {
                dst[y] = Self::update_row(src[y - 1], src[y], src[y + 1]);
            }
            std::mem::swap(&mut src, &mut dst);
        }

        let arr: [u16; 8] = src[4..12].try_into().unwrap();
        self.mem.find_leaf_from_rows(arr.map(|x| (x >> 4) as u8))
    }

    /// `size_log2` is related to `nw`, `ne`, `sw`, `se` and return value
    #[inline(never)]
    fn update_nodes_single(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        size_log2: u32,
    ) -> NodeIdx {
        let [nwnw, nwne, nwsw, nwse] = {
            let n = self.mem.get(nw, size_log2);
            [n.nw, n.ne, n.sw, n.se]
        };
        let [nenw, nene, nesw, nese] = {
            let n = self.mem.get(ne, size_log2);
            [n.nw, n.ne, n.sw, n.se]
        };
        let [swnw, swne, swsw, swse] = {
            let n = self.mem.get(sw, size_log2);
            [n.nw, n.ne, n.sw, n.se]
        };
        let [senw, sene, sesw, sese] = {
            let n = self.mem.get(se, size_log2);
            [n.nw, n.ne, n.sw, n.se]
        };

        let [t00, t01, t02, t10, t11, t12, t20, t21, t22] = if size_log2 >= LEAF_SIZE_LOG2 + 2 {
            [
                self.mem.find_node(
                    self.mem.get(nwnw, size_log2 - 1).se,
                    self.mem.get(nwne, size_log2 - 1).sw,
                    self.mem.get(nwsw, size_log2 - 1).ne,
                    self.mem.get(nwse, size_log2 - 1).nw,
                    size_log2 - 1,
                ),
                self.mem.find_node(
                    self.mem.get(nwne, size_log2 - 1).se,
                    self.mem.get(nenw, size_log2 - 1).sw,
                    self.mem.get(nwse, size_log2 - 1).ne,
                    self.mem.get(nesw, size_log2 - 1).nw,
                    size_log2 - 1,
                ),
                self.mem.find_node(
                    self.mem.get(nenw, size_log2 - 1).se,
                    self.mem.get(nene, size_log2 - 1).sw,
                    self.mem.get(nesw, size_log2 - 1).ne,
                    self.mem.get(nese, size_log2 - 1).nw,
                    size_log2 - 1,
                ),
                self.mem.find_node(
                    self.mem.get(nwsw, size_log2 - 1).se,
                    self.mem.get(nwse, size_log2 - 1).sw,
                    self.mem.get(swnw, size_log2 - 1).ne,
                    self.mem.get(swne, size_log2 - 1).nw,
                    size_log2 - 1,
                ),
                self.mem.find_node(
                    self.mem.get(nwse, size_log2 - 1).se,
                    self.mem.get(nesw, size_log2 - 1).sw,
                    self.mem.get(swne, size_log2 - 1).ne,
                    self.mem.get(senw, size_log2 - 1).nw,
                    size_log2 - 1,
                ),
                self.mem.find_node(
                    self.mem.get(nesw, size_log2 - 1).se,
                    self.mem.get(nese, size_log2 - 1).sw,
                    self.mem.get(senw, size_log2 - 1).ne,
                    self.mem.get(sene, size_log2 - 1).nw,
                    size_log2 - 1,
                ),
                self.mem.find_node(
                    self.mem.get(swnw, size_log2 - 1).se,
                    self.mem.get(swne, size_log2 - 1).sw,
                    self.mem.get(swsw, size_log2 - 1).ne,
                    self.mem.get(swse, size_log2 - 1).nw,
                    size_log2 - 1,
                ),
                self.mem.find_node(
                    self.mem.get(swne, size_log2 - 1).se,
                    self.mem.get(senw, size_log2 - 1).sw,
                    self.mem.get(swse, size_log2 - 1).ne,
                    self.mem.get(sesw, size_log2 - 1).nw,
                    size_log2 - 1,
                ),
                self.mem.find_node(
                    self.mem.get(senw, size_log2 - 1).se,
                    self.mem.get(sene, size_log2 - 1).sw,
                    self.mem.get(sesw, size_log2 - 1).ne,
                    self.mem.get(sese, size_log2 - 1).nw,
                    size_log2 - 1,
                ),
            ]
        } else {
            [
                self.mem.find_leaf_from_parts(
                    self.mem.get(nwnw, LEAF_SIZE_LOG2).leaf_se(),
                    self.mem.get(nwne, LEAF_SIZE_LOG2).leaf_sw(),
                    self.mem.get(nwsw, LEAF_SIZE_LOG2).leaf_ne(),
                    self.mem.get(nwse, LEAF_SIZE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(nwne, LEAF_SIZE_LOG2).leaf_se(),
                    self.mem.get(nenw, LEAF_SIZE_LOG2).leaf_sw(),
                    self.mem.get(nwse, LEAF_SIZE_LOG2).leaf_ne(),
                    self.mem.get(nesw, LEAF_SIZE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(nenw, LEAF_SIZE_LOG2).leaf_se(),
                    self.mem.get(nene, LEAF_SIZE_LOG2).leaf_sw(),
                    self.mem.get(nesw, LEAF_SIZE_LOG2).leaf_ne(),
                    self.mem.get(nese, LEAF_SIZE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(nwsw, LEAF_SIZE_LOG2).leaf_se(),
                    self.mem.get(nwse, LEAF_SIZE_LOG2).leaf_sw(),
                    self.mem.get(swnw, LEAF_SIZE_LOG2).leaf_ne(),
                    self.mem.get(swne, LEAF_SIZE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(nwse, LEAF_SIZE_LOG2).leaf_se(),
                    self.mem.get(nesw, LEAF_SIZE_LOG2).leaf_sw(),
                    self.mem.get(swne, LEAF_SIZE_LOG2).leaf_ne(),
                    self.mem.get(senw, LEAF_SIZE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(nesw, LEAF_SIZE_LOG2).leaf_se(),
                    self.mem.get(nese, LEAF_SIZE_LOG2).leaf_sw(),
                    self.mem.get(senw, LEAF_SIZE_LOG2).leaf_ne(),
                    self.mem.get(sene, LEAF_SIZE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(swnw, LEAF_SIZE_LOG2).leaf_se(),
                    self.mem.get(swne, LEAF_SIZE_LOG2).leaf_sw(),
                    self.mem.get(swsw, LEAF_SIZE_LOG2).leaf_ne(),
                    self.mem.get(swse, LEAF_SIZE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(swne, LEAF_SIZE_LOG2).leaf_se(),
                    self.mem.get(senw, LEAF_SIZE_LOG2).leaf_sw(),
                    self.mem.get(swse, LEAF_SIZE_LOG2).leaf_ne(),
                    self.mem.get(sesw, LEAF_SIZE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(senw, LEAF_SIZE_LOG2).leaf_se(),
                    self.mem.get(sene, LEAF_SIZE_LOG2).leaf_sw(),
                    self.mem.get(sesw, LEAF_SIZE_LOG2).leaf_ne(),
                    self.mem.get(sese, LEAF_SIZE_LOG2).leaf_nw(),
                ),
            ]
        };
        let q00 = self.mem.find_node(t00, t01, t10, t11, size_log2);
        let q01 = self.mem.find_node(t01, t02, t11, t12, size_log2);
        let q10 = self.mem.find_node(t10, t11, t20, t21, size_log2);
        let q11 = self.mem.find_node(t11, t12, t21, t22, size_log2);

        let [s00, s01, s10, s11] = [q00, q01, q10, q11].map(|x| self.update_node(x, size_log2));

        self.mem.find_node(s00, s01, s10, s11, size_log2)
    }

    /// `size_log2` is related to `nw`, `ne`, `sw`, `se` and return value
    #[inline(never)]
    fn update_nodes_double(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        size_log2: u32,
    ) -> NodeIdx {
        let [nw_, ne_, sw_, se_] = [nw, ne, sw, se].map(|x| self.mem.get(x, size_log2));

        // First stage
        let p11 = PrefetchedNode::new(&self.mem, nw_.se, ne_.sw, sw_.ne, se_.nw, size_log2);
        let p01 = PrefetchedNode::new(&self.mem, nw_.ne, ne_.nw, nw_.se, ne_.sw, size_log2);
        let p12 = PrefetchedNode::new(&self.mem, ne_.sw, ne_.se, se_.nw, se_.ne, size_log2);
        let p10 = PrefetchedNode::new(&self.mem, nw_.sw, nw_.se, sw_.nw, sw_.ne, size_log2);
        let p21 = PrefetchedNode::new(&self.mem, sw_.ne, se_.nw, sw_.se, se_.sw, size_log2);

        let t00 = self.update_node(nw, size_log2);
        let t01 = self.update_node(p01.find(), size_log2);
        let t02 = self.update_node(ne, size_log2);
        let t12 = self.update_node(p12.find(), size_log2);
        let t11 = self.update_node(p11.find(), size_log2);
        let t10 = self.update_node(p10.find(), size_log2);
        let t20 = self.update_node(sw, size_log2);
        let t21 = self.update_node(p21.find(), size_log2);
        let t22 = self.update_node(se, size_log2);

        // Second stage
        let pse = PrefetchedNode::new(&self.mem, t11, t12, t21, t22, size_log2);
        let psw = PrefetchedNode::new(&self.mem, t10, t11, t20, t21, size_log2);
        let pnw = PrefetchedNode::new(&self.mem, t00, t01, t10, t11, size_log2);
        let pne = PrefetchedNode::new(&self.mem, t01, t02, t11, t12, size_log2);
        let t_se = self.update_node(pse.find(), size_log2);
        let t_sw = self.update_node(psw.find(), size_log2);
        let t_nw = self.update_node(pnw.find(), size_log2);
        let t_ne = self.update_node(pne.find(), size_log2);
        self.mem.find_node(t_nw, t_ne, t_sw, t_se, size_log2)
    }

    /// Recursively updates nodes in graph.
    ///
    /// `size_log2` is related to `node`
    pub(super) fn update_node(&mut self, node: NodeIdx, size_log2: u32) -> NodeIdx {
        let n = self.mem.get(node, size_log2);
        if n.has_cache {
            return n.cache;
        }
        debug_assert!(node != NodeIdx(0), "Empty nodes should've been cached");

        let both_stages = self.generations_per_update_log2 + 2 >= size_log2;
        let cache = if size_log2 == LEAF_SIZE_LOG2 + 1 {
            let steps = if both_stages {
                LEAF_SIZE / 2
            } else {
                1 << self.generations_per_update_log2
            };
            self.update_leaves(n.nw, n.ne, n.sw, n.se, steps)
        } else if both_stages {
            self.update_nodes_double(n.nw, n.ne, n.sw, n.se, size_log2 - 1)
        } else {
            self.update_nodes_single(n.nw, n.ne, n.sw, n.se, size_log2 - 1)
        };
        let n = self.mem.get_mut(node, size_log2);
        n.cache = cache;
        n.has_cache = true;
        cache
    }

    /// Add a frame around the field: if `self.topology` is Unbounded, frame is blank,
    /// and if `self.topology` is Torus, frame mirrors the field.
    /// The field becomes two times bigger.
    pub(super) fn with_frame(&mut self, idx: NodeIdx, size_log2: u32) -> NodeIdx {
        let n = self.mem.get(idx, size_log2).clone();
        let [nw, ne, sw, se] = match self.topology {
            Topology::Torus => [self.mem.find_node(n.se, n.sw, n.ne, n.nw, size_log2); 4],
            Topology::Unbounded => {
                let b = NodeIdx(0);
                [
                    self.mem.find_node(b, b, b, n.nw, size_log2),
                    self.mem.find_node(b, b, n.ne, b, size_log2),
                    self.mem.find_node(b, n.sw, b, b, size_log2),
                    self.mem.find_node(n.se, b, b, b, size_log2),
                ]
            }
        };
        self.mem.find_node(nw, ne, sw, se, size_log2 + 1)
    }

    /// Remove a frame around the field, making it two times smaller.
    pub(super) fn without_frame(&mut self, idx: NodeIdx, size_log2: u32) -> NodeIdx {
        let n = self.mem.get(idx, size_log2);
        let [nw, ne, sw, se] = [
            self.mem.get(n.nw, size_log2 - 1).clone(),
            self.mem.get(n.ne, size_log2 - 1).clone(),
            self.mem.get(n.sw, size_log2 - 1).clone(),
            self.mem.get(n.se, size_log2 - 1).clone(),
        ];
        self.mem
            .find_node(nw.se, ne.sw, sw.ne, se.nw, size_log2 - 1)
    }

    pub(super) fn has_blank_frame(&self) -> bool {
        if self.size_log2 <= LEAF_SIZE_LOG2 + 1 {
            return false;
        }
        let root = self.mem.get(self.root, self.size_log2);
        let [nw, ne, sw, se] = [
            self.mem.get(root.nw, self.size_log2 - 1).clone(),
            self.mem.get(root.ne, self.size_log2 - 1).clone(),
            self.mem.get(root.sw, self.size_log2 - 1).clone(),
            self.mem.get(root.se, self.size_log2 - 1).clone(),
        ];
        [
            nw.sw, nw.nw, nw.ne, ne.nw, ne.ne, ne.se, se.ne, se.se, se.sw, sw.se, sw.sw, sw.nw,
        ]
        .iter()
        .all(|&x| x == NodeIdx(0))
    }

    pub(super) fn add_frame(&mut self, dx: &mut BigInt, dy: &mut BigInt) {
        self.root = self.with_frame(self.root, self.size_log2);
        *dx += BigInt::from(1) << (self.size_log2 - 1);
        *dy += BigInt::from(1) << (self.size_log2 - 1);
        self.size_log2 += 1;
    }

    pub(super) fn pop_frame(&mut self, dx: &mut BigInt, dy: &mut BigInt) {
        self.root = self.without_frame(self.root, self.size_log2);
        *dx -= BigInt::from(1) << (self.size_log2 - 2);
        *dy -= BigInt::from(1) << (self.size_log2 - 2);
        self.size_log2 -= 1;
    }

    /// Recursively mark nodes to rescue them from garbage collection.
    pub(super) fn gc_mark(&mut self, idx: NodeIdx, size_log2: u32) {
        if idx == NodeIdx(0) {
            return;
        }

        self.mem.get_mut(idx, size_log2).gc_marked = true;
        if size_log2 == LEAF_SIZE_LOG2 {
            return;
        }

        let n = self.mem.get(idx, size_log2).clone();
        self.gc_mark(n.nw, size_log2 - 1);
        self.gc_mark(n.ne, size_log2 - 1);
        self.gc_mark(n.sw, size_log2 - 1);
        self.gc_mark(n.se, size_log2 - 1);
    }
}

impl<Meta: Clone + Default> GoLEngine for HashLifeEngine<Meta> {
    fn from_pattern(pattern: &Pattern, topology: Topology) -> Result<Self> {
        fn inner(
            idx: NodeIdx,
            pattern: &Pattern,
        ) -> NodeIdx {
            match pattern.get_node(idx) {
            }
        }

        todo!()
    }

    fn current_state(&self) -> Pattern {
        todo!()
    }

    fn update(&mut self, generations_log2: u32) -> [BigInt; 2] {
        if self.has_cache && self.generations_per_update_log2 != generations_log2 {
            self.run_gc(); // TODO: only invalid cache
        }

        self.has_cache = true;
        self.generations_per_update_log2 = generations_log2;

        let frames_cnt = (generations_log2 + 2).max(self.size_log2 + 1) - self.size_log2;
        let (mut dx, mut dy) = (BigInt::ZERO, BigInt::ZERO);
        for _ in 0..frames_cnt {
            self.add_frame(&mut dx, &mut dy);
        }

        self.root = self.update_node(self.root, self.size_log2);
        self.size_log2 -= 1;
        dx -= BigInt::from(1) << (self.size_log2 - 1);
        dy -= BigInt::from(1) << (self.size_log2 - 1);

        match self.topology {
            Topology::Torus => {
                for _ in 0..frames_cnt - 1 {
                    self.pop_frame(&mut dx, &mut dy);
                }
            }
            Topology::Unbounded => {
                while self.has_blank_frame() {
                    self.pop_frame(&mut dx, &mut dy);
                }
            }
        }

        [dx, dy]
    }

    fn run_gc(&mut self) {
        self.gc_mark(self.root, self.size_log2);
        self.mem.gc_finish();
    }

    fn bytes_total(&self) -> usize {
        self.mem.bytes_total()
    }

    fn statistics(&mut self) -> String {
        let mut s = "Engine: Hashlife\n".to_string();
        s += &format!("Side length: 2^{}\n", self.size_log2);
        s += &self.mem.stats_fast();
        s
    }
}
