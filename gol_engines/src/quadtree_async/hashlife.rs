use super::{MemoryManager, NodeIdx, PopulationManager, LEAF_SIDE, LEAF_SIDE_LOG2};
use crate::{parse_rle, AsyncEngine, NiceInt, Topology, MAX_SIDE_LOG2, MIN_SIDE_LOG2};
use ahash::AHashMap as HashMap;

/// Implementation of [HashLife algorithm](https://conwaylife.com/wiki/HashLife)
pub struct HashLifeEngineAsync {
    size_log2: u32,
    root: NodeIdx,
    steps_per_update_log2: u32,
    has_cache: bool,
    mem: MemoryManager,
    population: PopulationManager,
}

impl HashLifeEngineAsync {
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
    fn update_leaves(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        steps: u64,
    ) -> NodeIdx {
        let [nw, ne, sw, se] =
            [nw, ne, sw, se].map(|x| self.mem.get(x, LEAF_SIDE_LOG2).leaf_cells());

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

    fn nine_children_overlapping(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        size_log2: u32,
    ) -> [NodeIdx; 9] {
        let [nw_, ne_, sw_, se_] = [nw, ne, sw, se].map(|x| self.mem.get(x, size_log2).clone());
        [
            nw,
            self.mem
                .find_node(nw_.ne, ne_.nw, nw_.se, ne_.sw, size_log2),
            ne,
            self.mem
                .find_node(nw_.sw, nw_.se, sw_.nw, sw_.ne, size_log2),
            self.mem
                .find_node(nw_.se, ne_.sw, sw_.ne, se_.nw, size_log2),
            self.mem
                .find_node(ne_.sw, ne_.se, se_.nw, se_.ne, size_log2),
            sw,
            self.mem
                .find_node(sw_.ne, se_.nw, sw_.se, se_.sw, size_log2),
            se,
        ]
    }

    fn nine_children_disjoint(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        size_log2: u32,
    ) -> [NodeIdx; 9] {
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

        if size_log2 >= LEAF_SIDE_LOG2 + 2 {
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
                    self.mem.get(nwnw, LEAF_SIDE_LOG2).leaf_se(),
                    self.mem.get(nwne, LEAF_SIDE_LOG2).leaf_sw(),
                    self.mem.get(nwsw, LEAF_SIDE_LOG2).leaf_ne(),
                    self.mem.get(nwse, LEAF_SIDE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(nwne, LEAF_SIDE_LOG2).leaf_se(),
                    self.mem.get(nenw, LEAF_SIDE_LOG2).leaf_sw(),
                    self.mem.get(nwse, LEAF_SIDE_LOG2).leaf_ne(),
                    self.mem.get(nesw, LEAF_SIDE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(nenw, LEAF_SIDE_LOG2).leaf_se(),
                    self.mem.get(nene, LEAF_SIDE_LOG2).leaf_sw(),
                    self.mem.get(nesw, LEAF_SIDE_LOG2).leaf_ne(),
                    self.mem.get(nese, LEAF_SIDE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(nwsw, LEAF_SIDE_LOG2).leaf_se(),
                    self.mem.get(nwse, LEAF_SIDE_LOG2).leaf_sw(),
                    self.mem.get(swnw, LEAF_SIDE_LOG2).leaf_ne(),
                    self.mem.get(swne, LEAF_SIDE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(nwse, LEAF_SIDE_LOG2).leaf_se(),
                    self.mem.get(nesw, LEAF_SIDE_LOG2).leaf_sw(),
                    self.mem.get(swne, LEAF_SIDE_LOG2).leaf_ne(),
                    self.mem.get(senw, LEAF_SIDE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(nesw, LEAF_SIDE_LOG2).leaf_se(),
                    self.mem.get(nese, LEAF_SIDE_LOG2).leaf_sw(),
                    self.mem.get(senw, LEAF_SIDE_LOG2).leaf_ne(),
                    self.mem.get(sene, LEAF_SIDE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(swnw, LEAF_SIDE_LOG2).leaf_se(),
                    self.mem.get(swne, LEAF_SIDE_LOG2).leaf_sw(),
                    self.mem.get(swsw, LEAF_SIDE_LOG2).leaf_ne(),
                    self.mem.get(swse, LEAF_SIDE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(swne, LEAF_SIDE_LOG2).leaf_se(),
                    self.mem.get(senw, LEAF_SIDE_LOG2).leaf_sw(),
                    self.mem.get(swse, LEAF_SIDE_LOG2).leaf_ne(),
                    self.mem.get(sesw, LEAF_SIDE_LOG2).leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    self.mem.get(senw, LEAF_SIDE_LOG2).leaf_se(),
                    self.mem.get(sene, LEAF_SIDE_LOG2).leaf_sw(),
                    self.mem.get(sesw, LEAF_SIDE_LOG2).leaf_ne(),
                    self.mem.get(sese, LEAF_SIDE_LOG2).leaf_nw(),
                ),
            ]
        }
    }

    fn four_children_overlapping(&mut self, arr: &[NodeIdx; 9], size_log2: u32) -> [NodeIdx; 4] {
        [
            self.mem
                .find_node(arr[0], arr[1], arr[3], arr[4], size_log2),
            self.mem
                .find_node(arr[1], arr[2], arr[4], arr[5], size_log2),
            self.mem
                .find_node(arr[3], arr[4], arr[6], arr[7], size_log2),
            self.mem
                .find_node(arr[4], arr[5], arr[7], arr[8], size_log2),
        ]
    }

    /// Recursively updates nodes in graph.
    ///
    /// `size_log2` is related to `node`
    fn update_node(&mut self, node: NodeIdx, size_log2: u32) -> NodeIdx {
        let n = self.mem.get(node, size_log2);
        if n.has_cache {
            return n.cache;
        }
        debug_assert!(node != NodeIdx(0), "Empty nodes should've been cached");

        let both_stages = self.steps_per_update_log2 + 2 >= size_log2;
        let cache = if size_log2 == LEAF_SIDE_LOG2 + 1 {
            let steps = if both_stages {
                LEAF_SIDE / 2
            } else {
                1 << self.steps_per_update_log2
            };
            self.update_leaves(n.nw, n.ne, n.sw, n.se, steps)
        } else if both_stages {
            let mut arr9 = self.nine_children_overlapping(n.nw, n.ne, n.sw, n.se, size_log2 - 1);
            for x in &mut arr9 {
                *x = self.update_node(*x, size_log2 - 1);
            }

            let mut arr4 = self.four_children_overlapping(&arr9, size_log2 - 1);
            for x in &mut arr4 {
                *x = self.update_node(*x, size_log2 - 1);
            }
            let [nw, ne, sw, se] = arr4;
            self.mem.find_node(nw, ne, sw, se, size_log2 - 1)
        } else {
            let arr9 = self.nine_children_disjoint(n.nw, n.ne, n.sw, n.se, size_log2 - 1);

            let mut arr4 = self.four_children_overlapping(&arr9, size_log2 - 1);
            for x in &mut arr4 {
                *x = self.update_node(*x, size_log2 - 1);
            }
            let [nw, ne, sw, se] = arr4;
            self.mem.find_node(nw, ne, sw, se, size_log2 - 1)
        };
        let n = self.mem.get_mut(node, size_log2);
        n.cache = cache;
        n.has_cache = true;
        cache
    }

    /// Add a frame around the field: if `topology` is Unbounded, frame is blank,
    /// and if `topology` is Torus, frame mirrors the field.
    /// The field becomes two times bigger.
    fn with_frame(&mut self, idx: NodeIdx, size_log2: u32, topology: Topology) -> NodeIdx {
        let n = self.mem.get(idx, size_log2).clone();
        let [nw, ne, sw, se] = match topology {
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
    fn without_frame(&mut self, idx: NodeIdx, size_log2: u32) -> NodeIdx {
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

    fn frame_is_blank(&self) -> bool {
        let root = self.mem.get(self.root, self.size_log2);
        let [nw, ne, sw, se] = [
            self.mem.get(root.nw, self.size_log2 - 1).clone(),
            self.mem.get(root.ne, self.size_log2 - 1).clone(),
            self.mem.get(root.sw, self.size_log2 - 1).clone(),
            self.mem.get(root.se, self.size_log2 - 1).clone(),
        ];
        self.size_log2 > MIN_SIDE_LOG2
            && nw.sw == NodeIdx(0)
            && nw.nw == NodeIdx(0)
            && nw.ne == NodeIdx(0)
            && ne.nw == NodeIdx(0)
            && ne.ne == NodeIdx(0)
            && ne.se == NodeIdx(0)
            && se.ne == NodeIdx(0)
            && se.se == NodeIdx(0)
            && se.sw == NodeIdx(0)
            && sw.se == NodeIdx(0)
            && sw.sw == NodeIdx(0)
            && sw.nw == NodeIdx(0)
    }

    fn add_frame(&mut self, topology: Topology, dx: &mut u64, dy: &mut u64) {
        self.root = self.with_frame(self.root, self.size_log2, topology);
        *dx += 1 << (self.size_log2 - 1);
        *dy += 1 << (self.size_log2 - 1);
        self.size_log2 += 1;
        assert!(self.size_log2 <= MAX_SIDE_LOG2);
    }

    fn pop_frame(&mut self, dx: &mut u64, dy: &mut u64) {
        self.root = self.without_frame(self.root, self.size_log2);
        *dx -= 1 << (self.size_log2 - 2);
        *dy -= 1 << (self.size_log2 - 2);
        self.size_log2 -= 1;
        assert!(self.size_log2 >= MIN_SIDE_LOG2);
    }

    /// Recursively mark nodes to rescue them from garbage collection.
    fn gc_mark(&mut self, idx: NodeIdx, size_log2: u32) {
        if idx == NodeIdx(0) {
            return;
        }

        self.mem.get_mut(idx, size_log2).gc_marked = true;
        if size_log2 == LEAF_SIDE_LOG2 {
            return;
        }

        let n = self.mem.get(idx, size_log2).clone();
        self.gc_mark(n.nw, size_log2 - 1);
        self.gc_mark(n.ne, size_log2 - 1);
        self.gc_mark(n.sw, size_log2 - 1);
        self.gc_mark(n.se, size_log2 - 1);
    }
}

impl AsyncEngine for HashLifeEngineAsync {
    fn blank(size_log2: u32) -> Self {
        assert!((MIN_SIDE_LOG2..=MAX_SIDE_LOG2).contains(&size_log2));
        let mut mem = MemoryManager::new();
        let root = mem.find_node(NodeIdx(0), NodeIdx(0), NodeIdx(0), NodeIdx(0), size_log2);
        Self {
            size_log2,
            root,
            steps_per_update_log2: 0,
            has_cache: false,
            mem,
            population: PopulationManager::new(),
        }
    }

    fn from_recursive_otca_metapixel(depth: u32, top_pattern: Vec<Vec<u8>>) -> Self {
        let k = top_pattern.len();
        assert!(
            top_pattern.iter().all(|row| row.len() == k),
            "Top pattern must be square"
        );
        assert!(k.is_power_of_two());

        const OTCA_SIZE: u64 = 2048;

        let otca_patterns = ["res/otca_0.rle", "res/otca_1.rle"].map(|path| {
            let buf = if let Ok(data) = std::fs::read(path) {
                data
            } else {
                std::fs::read(format!("../{}", path)).unwrap()
            };
            let (size_log2, data) = parse_rle(&buf);
            assert_eq!(1 << size_log2, OTCA_SIZE);
            data
        });

        if depth == 0 {
            panic!("Use `from_cells_array` instead");
        }

        let mut mem = MemoryManager::new();
        let (mut nodes_curr, mut nodes_next) = (vec![], vec![]);
        // creating first-level OTCA nodes
        let mut otca_nodes = [0, 1].map(|i| {
            for y in 0..OTCA_SIZE / LEAF_SIDE {
                for x in 0..OTCA_SIZE / LEAF_SIDE {
                    let mut data = [0; LEAF_SIDE as usize];
                    for sy in 0..LEAF_SIDE {
                        for sx in 0..LEAF_SIDE {
                            let pos = (sx + sy * LEAF_SIDE) / LEAF_SIDE;
                            let mask = 1 << ((sx + sy * LEAF_SIDE) % LEAF_SIDE);
                            let idx =
                                ((sx + x * LEAF_SIDE) + (sy + y * LEAF_SIDE) * OTCA_SIZE) as usize;
                            if otca_patterns[i][idx / 64] & (1 << (idx % 64)) != 0 {
                                data[pos as usize] |= mask;
                            }
                        }
                    }
                    nodes_curr.push(mem.find_leaf_from_rows(data));
                }
            }
            let mut t = OTCA_SIZE / LEAF_SIDE;
            while t != 1 {
                for y in (0..t).step_by(2) {
                    for x in (0..t).step_by(2) {
                        let nw = nodes_curr[(x + y * t) as usize];
                        let ne = nodes_curr[((x + 1) + y * t) as usize];
                        let sw = nodes_curr[(x + (y + 1) * t) as usize];
                        let se = nodes_curr[((x + 1) + (y + 1) * t) as usize];
                        nodes_next.push(mem.find_node(
                            nw,
                            ne,
                            sw,
                            se,
                            OTCA_SIZE.ilog2() - t.ilog2() + 1,
                        ));
                    }
                }
                std::mem::swap(&mut nodes_curr, &mut nodes_next);
                nodes_next.clear();
                t >>= 1;
            }
            assert_eq!(nodes_curr.len(), 1);
            nodes_curr.pop().unwrap()
        });
        // creating next-levels OTCA nodes
        for d in 1..depth {
            let otca_nodes_next = [0, 1].map(|i| {
                for y in 0..OTCA_SIZE {
                    for x in 0..OTCA_SIZE {
                        let idx = (x + y * OTCA_SIZE) as usize;
                        let state = (otca_patterns[i][idx / 64] & (1 << (idx % 64)) != 0) as usize;
                        nodes_curr.push(otca_nodes[state]);
                    }
                }
                let mut t = OTCA_SIZE;
                while t != 1 {
                    for y in (0..t).step_by(2) {
                        for x in (0..t).step_by(2) {
                            let nw = nodes_curr[(x + y * t) as usize];
                            let ne = nodes_curr[((x + 1) + y * t) as usize];
                            let sw = nodes_curr[(x + (y + 1) * t) as usize];
                            let se = nodes_curr[((x + 1) + (y + 1) * t) as usize];
                            nodes_next.push(mem.find_node(
                                nw,
                                ne,
                                sw,
                                se,
                                (d + 1) * OTCA_SIZE.ilog2() - t.ilog2() + 1,
                            ));
                        }
                    }
                    std::mem::swap(&mut nodes_curr, &mut nodes_next);
                    nodes_next.clear();
                    t >>= 1;
                }
                assert_eq!(nodes_curr.len(), 1);
                nodes_curr.pop().unwrap()
            });
            otca_nodes = otca_nodes_next;
        }
        // creating field from `top_pattern` using top-level OTCA nodes
        for row in top_pattern {
            for state in row {
                assert!(state == 0 || state == 1);
                let state = state as usize;
                nodes_curr.push(otca_nodes[state]);
            }
        }
        let mut t = k;
        while t != 1 {
            for y in (0..t).step_by(2) {
                for x in (0..t).step_by(2) {
                    let nw = nodes_curr[x + y * t];
                    let ne = nodes_curr[(x + 1) + y * t];
                    let sw = nodes_curr[x + (y + 1) * t];
                    let se = nodes_curr[(x + 1) + (y + 1) * t];
                    nodes_next.push(mem.find_node(
                        nw,
                        ne,
                        sw,
                        se,
                        depth * OTCA_SIZE.ilog2() + k.ilog2() - t.ilog2() + 1,
                    ));
                }
            }
            std::mem::swap(&mut nodes_curr, &mut nodes_next);
            nodes_next.clear();
            t >>= 1;
        }
        assert_eq!(nodes_curr.len(), 1);
        let root = nodes_curr.pop().unwrap();

        let size_log2 = OTCA_SIZE.ilog2() * depth + k.ilog2();
        assert!((MIN_SIDE_LOG2..=MAX_SIDE_LOG2).contains(&size_log2));
        Self {
            size_log2,
            root,
            mem,
            ..Default::default()
        }
    }

    fn from_macrocell(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        let mut mem = MemoryManager::new();
        let mut codes: HashMap<usize, NodeIdx> = HashMap::new();
        codes.insert(0, NodeIdx(0));
        let mut last_node = None;
        let mut size_log2 = 0;

        for s in data
            .split(|&x| x == b'\n')
            .skip(1)
            .filter(|&s| !s.is_empty() && s[0] != b'#')
        {
            let node = if s[0].is_ascii_digit() {
                // non-leaf
                let mut iter = s.split(|&x| x == b' ');
                let [k, nw, ne, sw, se] = [0; 5].map(|_| {
                    std::str::from_utf8(iter.next().unwrap())
                        .unwrap()
                        .parse::<usize>()
                        .unwrap()
                });
                size_log2 = k as u32;
                assert!((LEAF_SIDE_LOG2 + 1..=MAX_SIDE_LOG2).contains(&size_log2));
                let [nw, ne, sw, se] = [nw, ne, sw, se].map(|x| {
                    codes
                        .get(&x)
                        .copied()
                        .unwrap_or_else(|| panic!("Node with code {} not found", x))
                });
                mem.find_node(nw, ne, sw, se, size_log2)
            } else {
                // is leaf
                let mut cells = 0u64;
                let (mut i, mut j) = (0, 0);
                for &c in s {
                    match c {
                        b'$' => (i, j) = (i + 1, 0),
                        b'*' => {
                            cells |= 1 << (i * 8 + j);
                            j += 1;
                            assert!(j <= 8);
                        }
                        b'.' => {
                            j += 1;
                            assert!(j <= 8);
                        }
                        _ => panic!("Unexpected symbol"),
                    }
                }
                assert!(i <= 8);
                mem.find_leaf_from_u64(cells)
            };
            codes.insert(codes.len(), node);
            last_node = Some(node);
        }
        assert!((MIN_SIDE_LOG2..=MAX_SIDE_LOG2).contains(&size_log2));
        Self {
            size_log2,
            root: last_node.unwrap(),
            mem,
            ..Default::default()
        }
    }

    fn from_cells_array(size_log2: u32, cells: Vec<u64>) -> Self {
        assert!((MIN_SIDE_LOG2..=MAX_SIDE_LOG2).contains(&size_log2));
        assert_eq!(cells.len(), 1 << (size_log2 * 2 - 6));
        let mut mem = MemoryManager::new();
        let (mut nodes_curr, mut nodes_next) = (vec![], vec![]);
        let n = 1 << size_log2;

        for y in 0..n / LEAF_SIDE {
            for x in 0..n / LEAF_SIDE {
                let mut data = [0; LEAF_SIDE as usize];
                for sy in 0..LEAF_SIDE {
                    for sx in 0..LEAF_SIDE {
                        let pos = (sx + sy * LEAF_SIDE) / LEAF_SIDE;
                        let mask = 1 << ((sx + sy * LEAF_SIDE) % LEAF_SIDE);
                        let idx = ((sx + x * LEAF_SIDE) + (sy + y * LEAF_SIDE) * n) as usize;
                        if cells[idx / 64] & (1 << (idx % 64)) != 0 {
                            data[pos as usize] |= mask;
                        }
                    }
                }
                nodes_curr.push(mem.find_leaf_from_rows(data));
            }
        }
        let mut t = n / LEAF_SIDE;
        while t != 1 {
            for y in (0..t).step_by(2) {
                for x in (0..t).step_by(2) {
                    let nw = nodes_curr[(x + y * t) as usize];
                    let ne = nodes_curr[((x + 1) + y * t) as usize];
                    let sw = nodes_curr[(x + (y + 1) * t) as usize];
                    let se = nodes_curr[((x + 1) + (y + 1) * t) as usize];
                    nodes_next.push(mem.find_node(nw, ne, sw, se, size_log2 - t.ilog2() + 1));
                }
            }
            std::mem::swap(&mut nodes_curr, &mut nodes_next);
            nodes_next.clear();
            t >>= 1;
        }
        assert_eq!(nodes_curr.len(), 1);
        let root = nodes_curr.pop().unwrap();
        Self {
            size_log2,
            root,
            mem,
            ..Default::default()
        }
    }

    fn save_as_macrocell(&mut self) -> Vec<u8> {
        #[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
        struct Key {
            size_log2: u32,
            idx: NodeIdx,
        }

        fn inner(
            idx: NodeIdx,
            size_log2: u32,
            mem: &MemoryManager,
            codes: &mut HashMap<Key, usize>,
            result: &mut Vec<String>,
        ) {
            if codes.contains_key(&Key { idx, size_log2 }) {
                return;
            }
            let n = mem.get(idx, size_log2);
            let mut s = String::new();
            if size_log2 == LEAF_SIDE_LOG2 {
                let data = n.leaf_cells();
                for t in data.iter() {
                    for i in 0..8 {
                        if t >> i & 1 != 0 {
                            s.push('*');
                        } else {
                            s.push('.');
                        }
                    }
                    while s.ends_with('.') {
                        s.pop();
                    }
                    s.push('$');
                }
            } else {
                inner(n.nw, size_log2 - 1, mem, codes, result);
                inner(n.ne, size_log2 - 1, mem, codes, result);
                inner(n.sw, size_log2 - 1, mem, codes, result);
                inner(n.se, size_log2 - 1, mem, codes, result);
                s = format!(
                    "{} {} {} {} {}",
                    size_log2,
                    codes
                        .get(&Key {
                            idx: n.nw,
                            size_log2: size_log2 - 1
                        })
                        .unwrap(),
                    codes
                        .get(&Key {
                            idx: n.ne,
                            size_log2: size_log2 - 1
                        })
                        .unwrap(),
                    codes
                        .get(&Key {
                            idx: n.sw,
                            size_log2: size_log2 - 1
                        })
                        .unwrap(),
                    codes
                        .get(&Key {
                            idx: n.se,
                            size_log2: size_log2 - 1
                        })
                        .unwrap(),
                );
            }
            let v = if idx != NodeIdx(0) {
                s.push('\n');
                result.push(s);
                result.len() - 1
            } else {
                0
            };
            codes.entry(Key { idx, size_log2 }).or_insert(v);
        }

        let mut codes = HashMap::new();
        let mut result = vec!["[M2] (conway)\n#R B3/S23\n".to_string()];
        inner(
            self.root,
            self.size_log2,
            &self.mem,
            &mut codes,
            &mut result,
        );

        result.iter().flat_map(|s| s.bytes()).collect()
    }

    fn get_cells(&self) -> Vec<u64> {
        fn inner(
            x: u64,
            y: u64,
            root_size: u64,
            size_log2: u32,
            node: NodeIdx,
            mem: &MemoryManager,
            result: &mut Vec<u64>,
        ) {
            if size_log2 == LEAF_SIDE_LOG2 {
                let mut idx = x + y * root_size;
                for row in mem.get(node, LEAF_SIDE_LOG2).leaf_cells() {
                    result[idx as usize / 64] |= (row as u64) << (idx % 64);
                    idx += root_size;
                }
            } else {
                let n = mem.get(node, size_log2);
                let size_log2 = size_log2 - 1;
                for (i, &child) in [n.nw, n.ne, n.sw, n.se].iter().enumerate() {
                    let x = x + (((i & 1 != 0) as u64) << size_log2);
                    let y = y + (((i & 2 != 0) as u64) << size_log2);
                    inner(x, y, root_size, size_log2, child, mem, result);
                }
            }
        }

        let mut result = vec![0; 1 << (self.size_log2 * 2 - 6)];
        inner(
            0,
            0,
            1 << self.size_log2,
            self.size_log2,
            self.root,
            &self.mem,
            &mut result,
        );
        result
    }

    fn side_length_log2(&self) -> u32 {
        self.size_log2
    }

    fn get_cell(&self, mut x: u64, mut y: u64) -> bool {
        let mut node = self.root;
        let mut size_log2 = self.size_log2;
        while size_log2 != LEAF_SIDE_LOG2 {
            let n = self.mem.get(node, size_log2);
            size_log2 -= 1;
            let size = 1 << size_log2;
            let idx = (x >= size) as usize + 2 * (y >= size) as usize;
            x -= ((x >= size) as u64) << size_log2;
            y -= ((y >= size) as u64) << size_log2;
            node = match idx {
                0 => n.nw,
                1 => n.ne,
                2 => n.sw,
                3 => n.se,
                _ => unreachable!(),
            };
        }
        self.mem.get(node, LEAF_SIDE_LOG2).leaf_cells()[y as usize] >> x & 1 != 0
    }

    fn set_cell(&mut self, x: u64, y: u64, state: bool) {
        fn inner(
            mut x: u64,
            mut y: u64,
            mut size_log2: u32,
            node: NodeIdx,
            state: bool,
            mem: &mut MemoryManager,
        ) -> NodeIdx {
            let n = mem.get(node, size_log2);
            if size_log2 == LEAF_SIDE_LOG2 {
                let mut data = n.leaf_cells();
                let mask = 1 << x;
                if state {
                    data[y as usize] |= mask;
                } else {
                    data[y as usize] &= !mask;
                }
                mem.find_leaf_from_rows(data)
            } else {
                let mut arr = [n.nw, n.ne, n.sw, n.se];
                size_log2 -= 1;
                let size = 1 << size_log2;
                let idx: usize = (x >= size) as usize + 2 * (y >= size) as usize;
                x -= (x >= size) as u64 * size;
                y -= (y >= size) as u64 * size;
                arr[idx] = inner(x, y, size_log2, arr[idx], state, mem);
                mem.find_node(arr[0], arr[1], arr[2], arr[3], size_log2 + 1)
            }
        }

        self.root = inner(x, y, self.size_log2, self.root, state, &mut self.mem);
    }

    fn update(&mut self, steps_log2: u32, topology: Topology) -> [u64; 2] {
        if self.has_cache && self.steps_per_update_log2 != steps_log2 {
            self.run_gc();
        }

        self.has_cache = true;
        self.steps_per_update_log2 = steps_log2;

        let frames_cnt = (steps_log2 + 2).max(self.size_log2 + 1) - self.size_log2;
        let (mut dx, mut dy) = (0, 0);
        for _ in 0..frames_cnt {
            self.add_frame(topology, &mut dx, &mut dy);
        }

        self.root = self.update_node(self.root, self.size_log2);
        self.size_log2 -= 1;
        dx -= 1 << (self.size_log2 - 1);
        dy -= 1 << (self.size_log2 - 1);

        match topology {
            Topology::Torus => {
                for _ in 0..frames_cnt - 1 {
                    self.pop_frame(&mut dx, &mut dy);
                }
            }
            Topology::Unbounded => {
                while self.frame_is_blank() {
                    self.pop_frame(&mut dx, &mut dy);
                }
            }
        }

        [dx, dy]
    }

    fn fill_texture(
        &mut self,
        viewport_x: &mut f64,
        viewport_y: &mut f64,
        size: &mut f64,
        resolution: &mut f64,
        dst: &mut Vec<f64>,
    ) {
        struct Args<'a> {
            node: NodeIdx,
            x: i64,
            y: i64,
            size_log2: u32,
            dst: &'a mut Vec<f64>,
            viewport_x: i64,
            viewport_y: i64,
            resolution: i64,
            viewport_size: i64,
            step_log2: u32,
            mem: &'a MemoryManager,
            population: &'a mut PopulationManager,
        }

        fn inner(args: &mut Args<'_>) {
            if args.step_log2 == args.size_log2 {
                let j = (args.x - args.viewport_x) >> args.step_log2;
                let i = (args.y - args.viewport_y) >> args.step_log2;
                args.dst[(j + i * args.resolution) as usize] =
                    args.population.get(args.node, args.size_log2, args.mem);
                return;
            }
            const LEAF_ISIZE: i64 = LEAF_SIDE as i64;
            let n = args.mem.get(args.node, args.size_log2);
            if args.size_log2 == LEAF_SIDE_LOG2 {
                let data = n.leaf_cells();
                let k = LEAF_ISIZE >> args.step_log2;
                let step = 1 << args.step_log2;
                for sy in 0..k {
                    for sx in 0..k {
                        let mut sum = 0;
                        for dy in 0..step {
                            for dx in 0..step {
                                let x = (sx * step + dx) % LEAF_ISIZE;
                                let y = (sy * step + dy) % LEAF_ISIZE;
                                let pos = (x + y * LEAF_ISIZE) / LEAF_ISIZE;
                                let offset = (x + y * LEAF_ISIZE) % LEAF_ISIZE;
                                sum += data[pos as usize] >> offset & 1;
                            }
                        }
                        let j = sx + ((args.x - args.viewport_x) >> args.step_log2);
                        let i = sy + ((args.y - args.viewport_y) >> args.step_log2);
                        args.dst[(j + i * args.resolution) as usize] = sum as f64;
                    }
                }
            } else {
                args.size_log2 -= 1;
                let half = 1 << args.size_log2;
                for (i, &child) in [n.nw, n.ne, n.sw, n.se].iter().enumerate() {
                    let mut x = args.x + half * (i & 1 != 0) as i64;
                    let mut y = args.y + half * (i & 2 != 0) as i64;
                    let mut node = child;
                    if x + half > args.viewport_x
                        && x < args.viewport_x + args.viewport_size
                        && y + half > args.viewport_y
                        && y < args.viewport_y + args.viewport_size
                    {
                        std::mem::swap(&mut x, &mut args.x);
                        std::mem::swap(&mut y, &mut args.y);
                        std::mem::swap(&mut node, &mut args.node);
                        inner(args);
                        std::mem::swap(&mut x, &mut args.x);
                        std::mem::swap(&mut y, &mut args.y);
                        std::mem::swap(&mut node, &mut args.node);
                    }
                }
                args.size_log2 += 1;
            }
        }

        let step_log2 = ((*size / *resolution) as u64).max(1).ilog2();
        let step: u64 = 1 << step_log2;
        let com_mul = step.max(LEAF_SIDE);
        let size_int = (*size as u64).next_multiple_of(com_mul) as i64 + com_mul as i64 * 2;
        *size = size_int as f64;
        let resolution_int = size_int / step as i64;
        *resolution = resolution_int as f64;
        let x_int = (*viewport_x as u64 + 1).next_multiple_of(com_mul) as i64 - com_mul as i64 * 2;
        *viewport_x = x_int as f64;
        let y_int = (*viewport_y as u64 + 1).next_multiple_of(com_mul) as i64 - com_mul as i64 * 2;
        *viewport_y = y_int as f64;

        dst.clear();
        dst.resize((resolution_int * resolution_int) as usize, 0.);
        if step_log2 > self.size_log2 {
            return;
        }

        let mut args = Args {
            node: self.root,
            x: 0,
            y: 0,
            size_log2: self.size_log2,
            dst,
            viewport_x: x_int,
            viewport_y: y_int,
            resolution: resolution_int,
            viewport_size: size_int,
            step_log2,
            mem: &self.mem,
            population: &mut self.population,
        };
        inner(&mut args);
    }

    fn population(&mut self) -> f64 {
        self.population.get(self.root, self.size_log2, &self.mem)
    }

    fn hash(&self) -> u64 {
        #[derive(Clone, PartialEq, Eq, Hash)]
        struct Key(NodeIdx, u32);

        fn inner(
            idx: NodeIdx,
            size_log2: u32,
            cache: &mut HashMap<Key, u64>,
            mem: &MemoryManager,
        ) -> u64 {
            if let Some(&val) = cache.get(&Key(idx, size_log2)) {
                return val;
            }

            let combine = |x: u64, y: u64| -> u64 {
                x ^ y
                    .wrapping_add(0x9e3779b9)
                    .wrapping_add(x << 6)
                    .wrapping_add(x >> 2)
            };

            let n = mem.get(idx, size_log2).clone();
            if size_log2 == LEAF_SIDE_LOG2 {
                u64::from_le_bytes(n.leaf_cells())
            } else {
                let mut result = 0;
                for x in [n.nw, n.ne, n.sw, n.se] {
                    result = combine(result, inner(x, size_log2 - 1, cache, mem));
                }
                cache.insert(Key(idx, size_log2), result);
                result
            }
        }

        let mut cache = HashMap::new();
        inner(self.root, self.size_log2, &mut cache, &self.mem)
    }

    fn bytes_total(&self) -> usize {
        self.mem.bytes_total() + self.population.bytes_total()
    }

    fn statistics(&mut self) -> String {
        let mut s = "Engine: Hashlife\n".to_string();
        s += &format!("Side length: 2^{}\n", self.size_log2);
        let (population, duration) = {
            let timer = std::time::Instant::now();
            let population = self.population();
            (population, timer.elapsed())
        };
        s += &format!("Population: {}\n", NiceInt::from_f64(population));
        s += &format!("Population compute time: {}\n", duration.as_secs_f64());
        s += &self.mem.stats_fast();
        s
    }

    fn run_gc(&mut self) {
        self.gc_mark(self.root, self.size_log2);
        self.mem.gc_finish();
        self.population.clear_cache();
    }
}

impl Default for HashLifeEngineAsync {
    fn default() -> Self {
        Self::blank(MIN_SIDE_LOG2)
    }
}
