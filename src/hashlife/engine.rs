use super::memory::{Manager, NodeIdx, QuadTreeNode};
use crate::{Engine, MAX_SIDE_LOG2, MIN_SIDE_LOG2};

const LEAF_SIZE: u64 = 8;

pub struct HashLifeEngine {
    n: u64,
    root: NodeIdx,
    steps_per_update_log2: u32,
    mem: Manager,
}

impl HashLifeEngine {
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

    #[inline(never)]
    fn update_leaves(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        steps: u64,
    ) -> NodeIdx {
        let [nw, ne, sw, se] = [nw, ne, sw, se].map(|x| self.mem.get(x).leaf_cells());

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
        self.mem.find_leaf(arr.map(|x| (x >> 4) as u8))
    }

    // It can be significantly optimized, but update_nodes_double is much slower anyway.
    fn update_nodes_single(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        size_log2: u32,
    ) -> NodeIdx {
        let nwne = self.mem.get(nw).ne;
        let nwsw = self.mem.get(nw).sw;
        let nwse = self.mem.get(nw).se;

        let nenw = self.mem.get(ne).nw;
        let nesw = self.mem.get(ne).sw;
        let nese = self.mem.get(ne).se;

        let swnw = self.mem.get(sw).nw;
        let swne = self.mem.get(sw).ne;
        let swse = self.mem.get(sw).se;

        let senw = self.mem.get(se).nw;
        let sene = self.mem.get(se).ne;
        let sesw = self.mem.get(se).sw;

        let t00 = {
            let temp = self.update_node(nw, size_log2);
            self.mem.get(temp).clone()
        };

        let t01 = {
            let node = self.mem.find_node(nwne, nenw, nwse, nesw);
            let temp = self.update_node(node, size_log2);
            self.mem.get(temp).clone()
        };
        let t10 = {
            let node = self.mem.find_node(nwsw, nwse, swnw, swne);
            let temp = self.update_node(node, size_log2);
            self.mem.get(temp).clone()
        };
        let t11 = {
            let node = self.mem.find_node(nwse, nesw, swne, senw);
            let temp = self.update_node(node, size_log2);
            self.mem.get(temp).clone()
        };
        let t02 = {
            let temp = self.update_node(ne, size_log2);
            self.mem.get(temp).clone()
        };
        let t12 = {
            let node = self.mem.find_node(nesw, nese, senw, sene);
            let temp = self.update_node(node, size_log2);
            self.mem.get(temp).clone()
        };
        let t20 = {
            let temp = self.update_node(sw, size_log2);
            self.mem.get(temp).clone()
        };
        let t21 = {
            let node = self.mem.find_node(swne, senw, swse, sesw);
            let temp = self.update_node(node, size_log2);
            self.mem.get(temp).clone()
        };
        let t22 = {
            let temp = self.update_node(se, size_log2);
            self.mem.get(temp).clone()
        };
        let [t_nw, t_ne, t_sw, t_se] = if size_log2 >= LEAF_SIZE.ilog2() + 2 {
            [
                self.mem.find_node(t00.se, t01.sw, t10.ne, t11.nw),
                self.mem.find_node(t01.se, t02.sw, t11.ne, t12.nw),
                self.mem.find_node(t10.se, t11.sw, t20.ne, t21.nw),
                self.mem.find_node(t11.se, t12.sw, t21.ne, t22.nw),
            ]
        } else {
            [
                self.mem.find_leaf_from_parts(
                    t00.leaf_se(),
                    t01.leaf_sw(),
                    t10.leaf_ne(),
                    t11.leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    t01.leaf_se(),
                    t02.leaf_sw(),
                    t11.leaf_ne(),
                    t12.leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    t10.leaf_se(),
                    t11.leaf_sw(),
                    t20.leaf_ne(),
                    t21.leaf_nw(),
                ),
                self.mem.find_leaf_from_parts(
                    t11.leaf_se(),
                    t12.leaf_sw(),
                    t21.leaf_ne(),
                    t22.leaf_nw(),
                ),
            ]
        };
        self.mem.find_node(t_nw, t_ne, t_sw, t_se)
    }

    #[cfg(not(feature = "prefetch"))]
    #[inline(never)]
    fn update_nodes_double(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        size_log2: u32,
    ) -> NodeIdx {
        let nwne = self.mem.get(nw).ne;
        let nwsw = self.mem.get(nw).sw;
        let nwse = self.mem.get(nw).se;

        let nenw = self.mem.get(ne).nw;
        let nesw = self.mem.get(ne).sw;
        let nese = self.mem.get(ne).se;

        let swnw = self.mem.get(sw).nw;
        let swne = self.mem.get(sw).ne;
        let swse = self.mem.get(sw).se;

        let senw = self.mem.get(se).nw;
        let sene = self.mem.get(se).ne;
        let sesw = self.mem.get(se).sw;

        // First stage
        let t11 = {
            let node = self.mem.find_node(nwse, nesw, swne, senw);
            self.update_node(node, size_log2)
        };
        let t00 = self.update_node(nw, size_log2);
        let t01 = {
            let node = self.mem.find_node(nwne, nenw, nwse, nesw);
            self.update_node(node, size_log2)
        };
        let t02 = self.update_node(ne, size_log2);
        let t12 = {
            let node = self.mem.find_node(nesw, nese, senw, sene);
            self.update_node(node, size_log2)
        };
        let t10 = {
            let node = self.mem.find_node(nwsw, nwse, swnw, swne);
            self.update_node(node, size_log2)
        };
        let t20 = self.update_node(sw, size_log2);
        let t21 = {
            let node = self.mem.find_node(swne, senw, swse, sesw);
            self.update_node(node, size_log2)
        };
        let t22 = self.update_node(se, size_log2);

        // Second stage
        let t_se = {
            let node = self.mem.find_node(t11, t12, t21, t22);
            self.update_node(node, size_log2)
        };
        let t_sw = {
            let node = self.mem.find_node(t10, t11, t20, t21);
            self.update_node(node, size_log2)
        };
        let t_nw = {
            let node = self.mem.find_node(t00, t01, t10, t11);
            self.update_node(node, size_log2)
        };
        let t_ne = {
            let node = self.mem.find_node(t01, t02, t11, t12);
            self.update_node(node, size_log2)
        };
        self.mem.find_node(t_nw, t_ne, t_sw, t_se)
    }

    #[cfg(feature = "prefetch")]
    #[inline(never)]
    fn update_nodes_double(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        size_log2: u32,
    ) -> NodeIdx {
        let [nw_, ne_, sw_, se_] = [nw, ne, sw, se].map(|x| self.mem.get(x));

        // First stage
        let su2 = self.mem.setup_prefetch(nw_.se, ne_.sw, sw_.ne, se_.nw);
        let su0 = self.mem.setup_prefetch(nw_.ne, ne_.nw, nw_.se, ne_.sw);
        let su1 = self.mem.setup_prefetch(ne_.sw, ne_.se, se_.nw, se_.ne);
        let su3 = self.mem.setup_prefetch(nw_.sw, nw_.se, sw_.nw, sw_.ne);
        let su4 = self.mem.setup_prefetch(sw_.ne, se_.nw, sw_.se, se_.sw);
        let t00 = self.update_node(nw, size_log2);

        let t01 = {
            let node = self.mem.find_node_prefetched(&su0);
            self.update_node(node, size_log2)
        };
        let t02 = self.update_node(ne, size_log2);
        let t12 = {
            let node = self.mem.find_node_prefetched(&su1);
            self.update_node(node, size_log2)
        };
        let t11 = {
            let node = self.mem.find_node_prefetched(&su2);
            self.update_node(node, size_log2)
        };
        let t10 = {
            let node = self.mem.find_node_prefetched(&su3);
            self.update_node(node, size_log2)
        };
        let t20 = self.update_node(sw, size_log2);
        let t21 = {
            let node = self.mem.find_node_prefetched(&su4);
            self.update_node(node, size_log2)
        };
        let t22 = self.update_node(se, size_log2);

        // Second stage
        let su5 = self.mem.setup_prefetch(t11, t12, t21, t22);
        let su1 = self.mem.setup_prefetch(t10, t11, t20, t21);
        let su2 = self.mem.setup_prefetch(t00, t01, t10, t11);
        let su3 = self.mem.setup_prefetch(t01, t02, t11, t12);
        let t_se = {
            let node = self.mem.find_node_prefetched(&su5);
            self.update_node(node, size_log2)
        };
        let t_sw = {
            let node = self.mem.find_node_prefetched(&su1);
            self.update_node(node, size_log2)
        };
        let t_nw = {
            let node = self.mem.find_node_prefetched(&su2);
            self.update_node(node, size_log2)
        };
        let t_ne = {
            let node = self.mem.find_node_prefetched(&su3);
            self.update_node(node, size_log2)
        };
        self.mem.find_node(t_nw, t_ne, t_sw, t_se)
    }

    fn update_node(&mut self, node: NodeIdx, mut size_log2: u32) -> NodeIdx {
        let n = self.mem.get(node);
        if !n.cache.is_null() {
            return n.cache;
        }

        size_log2 -= 1;

        let do_full_step = self.steps_per_update_log2 + 1 >= size_log2;
        let cache = if size_log2 == LEAF_SIZE.ilog2() {
            let steps = if do_full_step {
                LEAF_SIZE / 2
            } else {
                1 << self.steps_per_update_log2
            };
            self.update_leaves(n.nw, n.ne, n.sw, n.se, steps)
        } else if do_full_step {
            self.update_nodes_double(n.nw, n.ne, n.sw, n.se, size_log2)
        } else {
            self.update_nodes_single(n.nw, n.ne, n.sw, n.se, size_log2)
        };
        self.mem.get_mut(node).cache = cache;
        cache
    }

    /// Recursively builds OTCA megapixels `depth` times, uses `top_pattern` as the top level.
    ///
    /// If `depth` == 0, every cell is a regular cell, if 1 it is
    /// an OTCA build from regular cells and so on.
    ///
    /// `top_pattern` must consist of zeros and ones.
    pub fn from_recursive_otca_metapixel<const N: usize>(
        depth: u32,
        top_pattern: [[u8; N]; N],
    ) -> Self {
        assert!(N.is_power_of_two());

        const OTCA_SIZE: u64 = 2048;

        let otca_patterns = [
            include_bytes!("../../res/otca_0.rle").as_slice(),
            include_bytes!("../../res/otca_1.rle").as_slice(),
        ]
        .map(|buf| {
            let (n_log2, data) = crate::parse_rle(buf);
            assert_eq!(1 << n_log2, OTCA_SIZE);
            data
        });

        if depth == 0 {
            // TODO
            unimplemented!("Use `from_cells` instead");
        }

        let mut mem = Manager::new();
        let (mut nodes_curr, mut nodes_next) = (vec![], vec![]);
        // creating first-level OTCA nodes
        let mut otca_nodes = [0, 1].map(|i| {
            for y in 0..OTCA_SIZE / LEAF_SIZE {
                for x in 0..OTCA_SIZE / LEAF_SIZE {
                    let mut data = [0; LEAF_SIZE as usize];
                    for sy in 0..LEAF_SIZE {
                        for sx in 0..LEAF_SIZE {
                            let pos = (sx + sy * LEAF_SIZE) / LEAF_SIZE;
                            let mask = 1 << ((sx + sy * LEAF_SIZE) % LEAF_SIZE);
                            let idx =
                                ((sx + x * LEAF_SIZE) + (sy + y * LEAF_SIZE) * OTCA_SIZE) as usize;
                            if otca_patterns[i][idx / 64] & (1 << (idx % 64)) != 0 {
                                data[pos as usize] |= mask;
                            }
                        }
                    }
                    nodes_curr.push(mem.find_leaf(data));
                }
            }
            let mut t = OTCA_SIZE / LEAF_SIZE;
            while t != 1 {
                for y in (0..t).step_by(2) {
                    for x in (0..t).step_by(2) {
                        let nw = nodes_curr[(x + y * t) as usize];
                        let ne = nodes_curr[((x + 1) + y * t) as usize];
                        let sw = nodes_curr[(x + (y + 1) * t) as usize];
                        let se = nodes_curr[((x + 1) + (y + 1) * t) as usize];
                        nodes_next.push(mem.find_node(nw, ne, sw, se));
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
        for _ in 1..depth {
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
                            nodes_next.push(mem.find_node(nw, ne, sw, se));
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
                let state = state as usize;
                assert!(state == 0 || state == 1);
                nodes_curr.push(otca_nodes[state]);
            }
        }
        let mut t = N;
        while t != 1 {
            for y in (0..t).step_by(2) {
                for x in (0..t).step_by(2) {
                    let nw = nodes_curr[x + y * t];
                    let ne = nodes_curr[(x + 1) + y * t];
                    let sw = nodes_curr[x + (y + 1) * t];
                    let se = nodes_curr[(x + 1) + (y + 1) * t];
                    nodes_next.push(mem.find_node(nw, ne, sw, se));
                }
            }
            std::mem::swap(&mut nodes_curr, &mut nodes_next);
            nodes_next.clear();
            t >>= 1;
        }
        assert_eq!(nodes_curr.len(), 1);
        let root = nodes_curr.pop().unwrap();

        Self {
            n: OTCA_SIZE.pow(depth) * N as u64,
            root,
            steps_per_update_log2: 0,
            mem,
        }
    }
}

impl Engine for HashLifeEngine {
    fn blank(n_log2: u32) -> Self {
        assert!((MIN_SIDE_LOG2..=MAX_SIDE_LOG2).contains(&n_log2));
        let mut hashtable = Manager::new();
        let mut node = hashtable.find_leaf([0; LEAF_SIZE as usize]);
        for _ in LEAF_SIZE.ilog2()..n_log2 {
            node = hashtable.find_node(node, node, node, node);
        }
        Self {
            n: 1 << n_log2,
            root: node,
            steps_per_update_log2: 0,
            mem: hashtable,
        }
    }

    fn from_cells(n_log2: u32, cells: Vec<u64>) -> Self {
        assert_eq!(cells.len(), 1 << (n_log2 * 2 - 6));
        let mut hashtable = Manager::new();
        let (mut nodes_curr, mut nodes_next) = (vec![], vec![]);
        let n = 1 << n_log2;

        for y in 0..n / LEAF_SIZE {
            for x in 0..n / LEAF_SIZE {
                let mut data = [0; LEAF_SIZE as usize];
                for sy in 0..LEAF_SIZE {
                    for sx in 0..LEAF_SIZE {
                        let pos = (sx + sy * LEAF_SIZE) / LEAF_SIZE;
                        let mask = 1 << ((sx + sy * LEAF_SIZE) % LEAF_SIZE);
                        let idx = ((sx + x * LEAF_SIZE) + (sy + y * LEAF_SIZE) * n) as usize;
                        if cells[idx / 64] & (1 << (idx % 64)) != 0 {
                            data[pos as usize] |= mask;
                        }
                    }
                }
                nodes_curr.push(hashtable.find_leaf(data));
            }
        }
        let mut t = n / LEAF_SIZE;
        while t != 1 {
            for y in (0..t).step_by(2) {
                for x in (0..t).step_by(2) {
                    let nw = nodes_curr[(x + y * t) as usize];
                    let ne = nodes_curr[((x + 1) + y * t) as usize];
                    let sw = nodes_curr[(x + (y + 1) * t) as usize];
                    let se = nodes_curr[((x + 1) + (y + 1) * t) as usize];
                    nodes_next.push(hashtable.find_node(nw, ne, sw, se));
                }
            }
            std::mem::swap(&mut nodes_curr, &mut nodes_next);
            nodes_next.clear();
            t >>= 1;
        }
        assert_eq!(nodes_curr.len(), 1);
        let root = nodes_curr.pop().unwrap();
        Self {
            n,
            root,
            steps_per_update_log2: 0,
            mem: hashtable,
        }
    }

    fn save_to_file_mc(&self, path: &str) {
        use std::collections::HashMap;
        use std::fs::File;
        use std::io::Write;

        fn inner(
            node: NodeIdx,
            size_log2: u32,
            mem: &Manager,
            codes: &mut HashMap<NodeIdx, usize>,
            result: &mut Vec<String>,
        ) {
            if codes.contains_key(&node) {
                return;
            }
            let n = mem.get(node);
            let mut s = String::new();
            if size_log2 == LEAF_SIZE.ilog2() {
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
                    codes.get(&n.nw).unwrap(),
                    codes.get(&n.ne).unwrap(),
                    codes.get(&n.sw).unwrap(),
                    codes.get(&n.se).unwrap(),
                );
            }
            let v = if mem.get(node).population != 0. {
                result.push(s);
                result.len()
            } else {
                0
            };
            codes.entry(node).or_insert(v);
        }

        let mut codes = HashMap::new();
        let mut result = vec![];
        inner(
            self.root,
            self.n.ilog2(),
            &self.mem,
            &mut codes,
            &mut result,
        );

        let mut file = File::create(path).unwrap();
        write!(file, "[M2] (hi)\n#R B3/S23\n").unwrap();
        for s in result {
            writeln!(file, "{}", s).unwrap();
        }
    }

    fn get_cells(&self) -> Vec<u64> {
        fn inner(
            x: u64,
            y: u64,
            curr_size: u64,
            root_size: u64,
            node: NodeIdx,
            mem: &Manager,
            result: &mut Vec<u64>,
        ) {
            if curr_size == LEAF_SIZE {
                let mut idx = x + y * root_size;
                for row in mem.get(node).leaf_cells() {
                    result[idx as usize / 64] |= (row as u64) << (idx % 64);
                    idx += root_size;
                }
            } else {
                let curr_size = curr_size / 2;
                let n = mem.get(node);
                for (i, &child) in [n.nw, n.ne, n.sw, n.se].iter().enumerate() {
                    let x = x + curr_size * (i & 1 != 0) as u64;
                    let y = y + curr_size * (i & 2 != 0) as u64;
                    inner(x, y, curr_size, root_size, child, mem, result);
                }
            }
        }

        let mut result = vec![0; (self.n * self.n / 64) as usize];
        inner(0, 0, self.n, self.n, self.root, &self.mem, &mut result);
        result
    }

    fn side_length_log2(&self) -> u32 {
        self.n.ilog2()
    }

    fn population(&self) -> f64 {
        self.mem.get(self.root).population
    }

    fn get_cell(&self, mut x: u64, mut y: u64) -> bool {
        let mut node = self.root;
        let mut size = self.n;
        while size >= LEAF_SIZE {
            let n = self.mem.get(node);
            if size == LEAF_SIZE {
                assert_eq!(size, LEAF_SIZE);
                let data = n.leaf_cells();
                return data[y as usize] >> x & 1 != 0;
            }
            size >>= 1;
            let idx = (x >= size) as usize + 2 * (y >= size) as usize;
            x -= (x >= size) as u64 * size;
            y -= (y >= size) as u64 * size;
            node = match idx {
                0 => n.nw,
                1 => n.ne,
                2 => n.sw,
                3 => n.se,
                _ => unreachable!(),
            };
        }
        unreachable!("Size is smaller than the leaf size")
    }

    fn set_cell(&mut self, x: u64, y: u64, state: bool) {
        fn inner(
            mut x: u64,
            mut y: u64,
            mut size: u64,
            node: NodeIdx,
            state: bool,
            mem: &mut Manager,
        ) -> NodeIdx {
            let n = mem.get(node);
            if size == LEAF_SIZE {
                let mut data = n.leaf_cells();
                let mask = 1 << x;
                if state {
                    data[y as usize] |= mask;
                } else {
                    data[y as usize] &= !mask;
                }
                mem.find_leaf(data)
            } else {
                let mut arr = [n.nw, n.ne, n.sw, n.se];
                size >>= 1;
                let idx: usize = (x >= size) as usize + 2 * (y >= size) as usize;
                x -= (x >= size) as u64 * size;
                y -= (y >= size) as u64 * size;
                arr[idx] = inner(x, y, size, arr[idx], state, mem);
                mem.find_node(arr[0], arr[1], arr[2], arr[3])
            }
        }

        self.root = inner(x, y, self.n, self.root, state, &mut self.mem);
    }

    fn update(&mut self, steps_log2: u32) {
        if self.steps_per_update_log2 != steps_log2 {
            self.steps_per_update_log2 = steps_log2;
            // todo!("implement changing steps per update");
        }
        let top = self.root;
        let size_log2 = self.n.ilog2();
        let q = {
            let temp = self.mem.find_node(top, top, top, top);
            let q = self.update_node(temp, size_log2 + 1);
            self.mem.get(q)
        };
        let [nw, ne, sw, se] = [q.nw, q.ne, q.sw, q.se];
        self.root = self.mem.find_node(se, sw, ne, nw);
    }

    fn fill_texture(
        &self,
        viewport_x: &mut f64,
        viewport_y: &mut f64,
        size: &mut f64,
        resolution: &mut f64,
        dst: &mut Vec<f64>,
    ) -> u32 {
        struct Args<'a> {
            curr_size_log2: u32,
            viewport_x: u64,
            viewport_y: u64,
            resolution: u64,
            viewport_size: u64,
            step_log2: u32,
            dst: &'a mut Vec<f64>,
            mem: &'a Manager,
        }
        fn inner(node: &QuadTreeNode, curr_x: u64, curr_y: u64, args: &mut Args) {
            if args.step_log2 == args.curr_size_log2 {
                let j = (curr_x - args.viewport_x) >> args.step_log2;
                let i = (curr_y - args.viewport_y) >> args.step_log2;
                args.dst[(j + i * args.resolution) as usize] = node.population;
                return;
            }
            if args.curr_size_log2 == LEAF_SIZE.ilog2() {
                let data = node.leaf_cells();
                let k = LEAF_SIZE >> args.step_log2;
                let step = 1 << args.step_log2;
                for sy in 0..k {
                    for sx in 0..k {
                        let mut sum = 0;
                        for dy in 0..step {
                            for dx in 0..step {
                                let x = (sx * step + dx) % LEAF_SIZE;
                                let y = (sy * step + dy) % LEAF_SIZE;
                                let pos = (x + y * LEAF_SIZE) / LEAF_SIZE;
                                let offset = (x + y * LEAF_SIZE) % LEAF_SIZE;
                                sum += data[pos as usize] >> offset & 1;
                            }
                        }
                        let j = sx + ((curr_x - args.viewport_x) >> args.step_log2);
                        let i = sy + ((curr_y - args.viewport_y) >> args.step_log2);
                        args.dst[(j + i * args.resolution) as usize] = sum as f64;
                    }
                }
            } else {
                args.curr_size_log2 -= 1;
                let half = 1 << args.curr_size_log2;
                for (i, &child) in [node.nw, node.ne, node.sw, node.se].iter().enumerate() {
                    let x = curr_x + half * (i & 1 != 0) as u64;
                    let y = curr_y + half * (i & 2 != 0) as u64;
                    let child = args.mem.get(child);
                    if x + half > args.viewport_x
                        && x < args.viewport_x + args.viewport_size
                        && y + half > args.viewport_y
                        && y < args.viewport_y + args.viewport_size
                    {
                        inner(child, x, y, args);
                    }
                }
                args.curr_size_log2 += 1;
            }
        }

        let step_log2 = ((*size / *resolution) as u64).max(1).ilog2();
        let step = 1 << step_log2;
        let com_mul = step.max(LEAF_SIZE);
        let size_int = (*size as u64).next_multiple_of(com_mul) + com_mul;
        *size = size_int as f64;
        let resolution_int = size_int / step;
        *resolution = resolution_int as f64;
        let x_int = (*viewport_x as u64 + 1).next_multiple_of(com_mul) - com_mul;
        *viewport_x = x_int as f64;
        let y_int = (*viewport_y as u64 + 1).next_multiple_of(com_mul) - com_mul;
        *viewport_y = y_int as f64;

        dst.clear();
        dst.resize((resolution_int * resolution_int) as usize, 0.);
        let mut args = Args {
            curr_size_log2: self.n.ilog2(),
            viewport_x: x_int,
            viewport_y: y_int,
            resolution: resolution_int,
            viewport_size: size_int,
            step_log2,
            dst,
            mem: &self.mem,
        };
        inner(self.mem.get(self.root), 0, 0, &mut args);
        step_log2
    }

    fn stats(&self, verbose: bool) -> String {
        format!(
            "Engine: Hashlife\nn: 2^{}\n{}",
            self.n.ilog2(),
            self.mem.stats(verbose)
        )
    }
}
