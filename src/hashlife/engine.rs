use crate::Engine;

use super::memory::{HashTable, QuadTreeNode};
use std::path::Path;

const BASE_SIZE: u64 = 8;
const CELLS_IN_CHUNK: u64 = 8;
const CHUNKS_IN_LEAF: u64 = BASE_SIZE * BASE_SIZE / CELLS_IN_CHUNK;

pub struct HashLifeEngine {
    root: *mut QuadTreeNode,
    n: u64,
    pub hashtable: HashTable,
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

    fn base_update(&mut self, node: *mut QuadTreeNode) -> *mut QuadTreeNode {
        const W: usize = (BASE_SIZE / CELLS_IN_CHUNK) as usize;
        const H: usize = (BASE_SIZE) as usize;

        let node = unsafe { &mut *node };
        let v0 = unsafe { ((*node.nw).ne as u64).to_le_bytes() };
        let v1 = unsafe { ((*node.ne).ne as u64).to_le_bytes() };
        let v2 = unsafe { ((*node.sw).ne as u64).to_le_bytes() };
        let v3 = unsafe { ((*node.se).ne as u64).to_le_bytes() };

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
                let row_prev = u16::from_le_bytes(row_prev.try_into().unwrap());
                let row_curr = &src[y * 2 * W..(y + 1) * 2 * W];
                let row_curr = u16::from_le_bytes(row_curr.try_into().unwrap());
                let row_next = &src[(y + 1) * 2 * W..(y + 2) * 2 * W];
                let row_next = u16::from_le_bytes(row_next.try_into().unwrap());
                let dst = &mut dst[y * 2 * W..(y + 1) * 2 * W];
                let x = Self::update_row(row_prev, row_curr, row_next).to_le_bytes();
                dst.copy_from_slice(&x);
            }
            std::mem::swap(&mut src, &mut dst);
        }

        let mut result = [0; W * H];
        for y in 0..H {
            let t = (y + 4) * 2;
            result[y] = (u16::from_le_bytes(src[t..t + 2].try_into().unwrap()) >> 4) as u8;
        }
        self.hashtable.find_leaf(u64::from_le_bytes(result))
    }

    fn update_composite_sequential(
        &mut self,
        node: *mut QuadTreeNode,
        mut size_log2: u32,
    ) -> *mut QuadTreeNode {
        let [nw, ne, sw, se] = unsafe { [(*node).nw, (*node).ne, (*node).sw, (*node).se] };
        let [_, ne0, sw0, se0] = unsafe { [(*nw).nw, (*nw).ne, (*nw).sw, (*nw).se] };
        let [nw1, _, sw1, se1] = unsafe { [(*ne).nw, (*ne).ne, (*ne).sw, (*ne).se] };
        let [nw2, ne2, _, se2] = unsafe { [(*sw).nw, (*sw).ne, (*sw).sw, (*sw).se] };
        let [nw3, ne3, sw3, _] = unsafe { [(*se).nw, (*se).ne, (*se).sw, (*se).se] };

        size_log2 -= 1;
        let p0 = self.update_node(nw, size_log2);
        let temp = self.hashtable.find_node(ne0, nw1, se0, sw1);
        let p1 = self.update_node(temp, size_log2);
        let p2 = self.update_node(ne, size_log2);
        let temp = self.hashtable.find_node(sw0, se0, nw2, ne2);
        let p3 = self.update_node(temp, size_log2);
        let temp = self.hashtable.find_node(se0, sw1, ne2, nw3);
        let p4 = self.update_node(temp, size_log2);
        let temp = self.hashtable.find_node(sw1, se1, nw3, ne3);
        let p5 = self.update_node(temp, size_log2);
        let p6 = self.update_node(sw, size_log2);
        let temp = self.hashtable.find_node(ne2, nw3, se2, sw3);
        let p7 = self.update_node(temp, size_log2);
        let p8 = self.update_node(se, size_log2);

        let temp = self.hashtable.find_node(p0, p1, p3, p4);
        let q0 = self.update_node(temp, size_log2);
        let temp = self.hashtable.find_node(p1, p2, p4, p5);
        let q1 = self.update_node(temp, size_log2);
        let temp = self.hashtable.find_node(p3, p4, p6, p7);
        let q2 = self.update_node(temp, size_log2);
        let temp = self.hashtable.find_node(p4, p5, p7, p8);
        let q3 = self.update_node(temp, size_log2);
        self.hashtable.find_node(q0, q1, q2, q3)
    }

    fn update_node(&mut self, node: *mut QuadTreeNode, curr_size_log2: u32) -> *mut QuadTreeNode {
        let cache = unsafe { (*node).cache };
        if !cache.is_null() {
            return cache;
        }
        let cache = if curr_size_log2 == BASE_SIZE.ilog2() + 1 {
            self.base_update(node)
        } else {
            self.update_composite_sequential(node, curr_size_log2)
        };
        unsafe { (*node).cache = cache };
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
        // dead and alive
        let otca_patterns = ["res/otca_0.rle", "res/otca_1.rle"].map(|path| {
            use std::fs::File;
            use std::io::Read;
            let mut buf = vec![];
            File::open(path).unwrap().read_to_end(&mut buf).unwrap();
            let (w, h, data) = crate::parse_rle(&buf);
            assert_eq!(w, OTCA_SIZE);
            assert_eq!(h, OTCA_SIZE);
            data
        });

        if depth == 0 {
            // TODO
            unimplemented!()
        }

        let mut hashtable = HashTable::new();
        let (mut nodes_curr, mut nodes_next) = (vec![], vec![]);
        // creating first-level OTCA nodes
        let mut otca_nodes = [0, 1].map(|i| {
            for y in 0..OTCA_SIZE / BASE_SIZE {
                for x in 0..OTCA_SIZE / BASE_SIZE {
                    let mut data = [0; CHUNKS_IN_LEAF as usize];
                    for sy in 0..BASE_SIZE {
                        for sx in 0..BASE_SIZE {
                            let pos = (sx + sy * BASE_SIZE) / CELLS_IN_CHUNK;
                            let mask = 1 << ((sx + sy * BASE_SIZE) % CELLS_IN_CHUNK);
                            if otca_patterns[i]
                                [((sx + x * BASE_SIZE) + (sy + y * BASE_SIZE) * OTCA_SIZE) as usize]
                            {
                                data[pos as usize] |= mask;
                            }
                        }
                    }
                    nodes_curr.push(hashtable.find_leaf(u64::from_le_bytes(data)));
                }
            }
            let mut t = OTCA_SIZE / BASE_SIZE;
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
            nodes_curr.pop().unwrap()
        });
        // creating next-levels OTCA nodes
        for _ in 1..depth {
            let otca_nodes_next = [0, 1].map(|i| {
                for y in 0..OTCA_SIZE {
                    for x in 0..OTCA_SIZE {
                        let state = otca_patterns[i][(x + y * OTCA_SIZE) as usize] as usize;
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
                            nodes_next.push(hashtable.find_node(nw, ne, sw, se));
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
        assert!(N.is_power_of_two());
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
            root,
            n: OTCA_SIZE.pow(depth) * N as u64,
            hashtable,
        }
    }

    pub fn into_mc<P: AsRef<Path>>(&self, path: P) {
        use std::collections::HashMap;
        use std::fs::File;
        use std::io::Write;

        fn inner(
            node: *mut QuadTreeNode,
            size_log2: u32,
            codes: &mut HashMap<*mut QuadTreeNode, usize>,
            result: &mut Vec<String>,
        ) {
            if codes.contains_key(&node) {
                return;
            }
            let mut s = String::new();
            if unsafe { (*node).nw.is_null() } {
                let data = unsafe { ((*node).ne as u64).to_le_bytes() };
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
                let [nw, ne, sw, se] = unsafe { [(*node).nw, (*node).ne, (*node).sw, (*node).se] };
                inner(nw, size_log2 - 1, codes, result);
                inner(ne, size_log2 - 1, codes, result);
                inner(sw, size_log2 - 1, codes, result);
                inner(se, size_log2 - 1, codes, result);
                s = format!(
                    "{} {} {} {} {}",
                    size_log2,
                    codes.get(&nw).unwrap(),
                    codes.get(&ne).unwrap(),
                    codes.get(&sw).unwrap(),
                    codes.get(&se).unwrap(),
                );
            }
            let v = if unsafe { (*node).population } != 0.0 {
                result.push(s);
                result.len()
            } else {
                0
            };
            codes.entry(node).or_insert(v);
        }

        let mut codes = HashMap::new();
        let mut result = vec![];
        inner(self.root, self.n.ilog2(), &mut codes, &mut result);

        let mut file = File::create(path).unwrap();
        write!(file, "[M2] (hi)\n#R B3/S23\n").unwrap();
        for s in result {
            writeln!(file, "{}", s).unwrap();
        }
    }
}

impl Engine for HashLifeEngine {
    fn blank(n_log2: u32) -> Self {
        assert!((7..64).contains(&n_log2));
        let mut hashtable = HashTable::new();
        let mut node = hashtable.find_leaf(0);
        for _ in BASE_SIZE.ilog2()..n_log2 {
            node = hashtable.find_node(node, node, node, node);
        }
        Self {
            root: node,
            n: 1 << n_log2,
            hashtable,
        }
    }

    fn parse_rle(data: &[u8]) -> Self {
        unimplemented!()
    }

    fn side_length_log2(&self) -> u32 {
        self.n.ilog2()
    }

    fn get_cell(&self, mut x: u64, mut y: u64) -> bool {
        let mut node = self.root;
        let mut size = self.n;
        while size >= BASE_SIZE {
            size >>= 1;
            let idx = (x >= size) as usize + 2 * (y >= size) as usize;
            if unsafe { (*node).nw.is_null() } {
                let data = unsafe { ((*node).ne as u64).to_le_bytes() };
                let pos = (x + y * BASE_SIZE) / CELLS_IN_CHUNK;
                let offset = (x + y * BASE_SIZE) % CELLS_IN_CHUNK;
                return data[pos as usize] >> offset & 1 != 0;
            } else {
                node = match idx {
                    0 => unsafe { (*node).nw },
                    1 => unsafe { (*node).ne },
                    2 => unsafe { (*node).sw },
                    3 => unsafe { (*node).se },
                    _ => unreachable!(),
                };
            }
            x -= (x >= size) as u64 * size;
            y -= (y >= size) as u64 * size;
        }
        unreachable!("Size is smaller than the base size, which is impossible")
    }

    fn set_cell(&mut self, x: u64, y: u64, state: bool) {
        fn inner(
            mut x: u64,
            mut y: u64,
            mut size: u64,
            node: *mut QuadTreeNode,
            state: bool,
            engine: &mut HashLifeEngine,
        ) -> *mut QuadTreeNode {
            size >>= 1;
            let idx: usize = (x >= size) as usize + 2 * (y >= size) as usize;
            if size == BASE_SIZE {
                let mut data = unsafe { ((*node).ne as u64).to_le_bytes() };
                let pos = (x + y * BASE_SIZE) / CELLS_IN_CHUNK;
                let mask = 1 << ((x + y * BASE_SIZE) % CELLS_IN_CHUNK);
                if state {
                    data[pos as usize] |= mask;
                } else {
                    data[pos as usize] &= !mask;
                }
                engine.hashtable.find_leaf(u64::from_le_bytes(data))
            } else {
                let mut arr = unsafe { [(*node).nw, (*node).ne, (*node).sw, (*node).se] };
                x -= (x >= size) as u64 * size;
                y -= (y >= size) as u64 * size;
                arr[idx] = inner(x, y, size, arr[idx], state, engine);
                engine.hashtable.find_node(arr[0], arr[1], arr[2], arr[3])
            }
        }

        self.root = inner(x, y, self.n, self.root, state, self);
    }

    fn update(&mut self, steps_log2: u32) {
        println!("Changing update step is not supported");
        let top = self.root;
        let size_log2 = self.n.ilog2();
        let q = {
            let temp = self.hashtable.find_node(top, top, top, top);
            self.update_node(temp, size_log2 + 1)
        };
        let [nw, ne, sw, se] = unsafe { [(*q).nw, (*q).ne, (*q).sw, (*q).se] };
        self.root = self.hashtable.find_node(se, sw, ne, nw);
    }

    fn fill_texture(
        &self,
        viewport_x: &mut f64,
        viewport_y: &mut f64,
        size: &mut f64,
        resolution: &mut f64,
        dst: &mut Vec<f64>,
    ) {
        struct Args<'a> {
            curr_size_log2: u32,
            viewport_x: u64,
            viewport_y: u64,
            resolution: u64,
            viewport_size: u64,
            step_log2: u32,
            dst: &'a mut Vec<f64>,
        }
        fn inner(node: &QuadTreeNode, curr_x: u64, curr_y: u64, args: &mut Args) {
            if args.step_log2 == args.curr_size_log2 {
                let j = (curr_x - args.viewport_x) >> args.step_log2;
                let i = (curr_y - args.viewport_y) >> args.step_log2;
                args.dst[(j + i * args.resolution) as usize] = node.population;
                return;
            }
            if node.nw.is_null() {
                let data = (node.ne as u64).to_le_bytes();
                let k = BASE_SIZE >> args.step_log2;
                let step = 1 << args.step_log2;
                for sy in 0..k {
                    for sx in 0..k {
                        let mut sum = 0;
                        for dy in 0..step {
                            for dx in 0..step {
                                let x = (sx * step + dx) & (BASE_SIZE - 1);
                                let y = (sy * step + dy) & (BASE_SIZE - 1);
                                let pos = (x + y * BASE_SIZE) / CELLS_IN_CHUNK;
                                let offset = (x + y * BASE_SIZE) % CELLS_IN_CHUNK;
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
                for (i, &child) in [node.nw, node.ne, node.sw, node.se]
                    .iter()
                    .enumerate()
                {
                    let x = curr_x + half * (i & 1 != 0) as u64;
                    let y = curr_y + half * (i & 2 != 0) as u64;
                    let child = unsafe { &*child };
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
        let com_mul = step.max(BASE_SIZE);
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
        };
        inner(unsafe { &*self.root }, 0, 0, &mut args);
    }

    fn stats(&self) -> String {
        format!(
            "
\tHashLifeEngine:
n: {}
{}",
            self.n,
            self.hashtable.stats()
        )
    }
}
