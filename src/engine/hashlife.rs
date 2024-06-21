use ahash::AHashMap;
use std::{cell::RefCell, path::Path, rc::Rc};

type Chunk = u8;

const BASE_SIZE: usize = 8;
const CELLS_IN_CHUNK: usize = std::mem::size_of::<Chunk>() * 8;
const CHUNKS_IN_LEAF: usize = BASE_SIZE * BASE_SIZE / CELLS_IN_CHUNK;

type HashMapNodesComposite = AHashMap<[usize; 4], Rc<QuadTreeNode>>;
type HashMapNodesLeaf = AHashMap<[Chunk; CHUNKS_IN_LEAF], Rc<QuadTreeNode>>;

enum NodeData {
    Composite([Rc<QuadTreeNode>; 4]),
    Leaf(Box<[Chunk; CHUNKS_IN_LEAF]>),
}

struct QuadTreeNode {
    population: f64,
    data: NodeData,
    next: RefCell<Option<Rc<QuadTreeNode>>>,
}

pub struct ConwayFieldHash256 {
    root: Rc<QuadTreeNode>,
    size: usize,
    nodes_composite: HashMapNodesComposite,
    nodes_leaf: HashMapNodesLeaf,
}

impl ConwayFieldHash256 {
    fn get_leaf_node(
        data: [Chunk; CHUNKS_IN_LEAF],
        nodes: &mut HashMapNodesLeaf,
    ) -> Rc<QuadTreeNode> {
        if let Some(node) = nodes.get(&data) {
            node.clone()
        } else {
            let result = Rc::new(QuadTreeNode {
                population: data.iter().map(|x| x.count_ones()).sum::<u32>() as f64,
                data: NodeData::Leaf(Box::new(data)),
                next: RefCell::new(None),
            });
            nodes.insert(data, result.clone());
            result
        }
    }

    fn get_composite_node(
        nw: &Rc<QuadTreeNode>,
        ne: &Rc<QuadTreeNode>,
        sw: &Rc<QuadTreeNode>,
        se: &Rc<QuadTreeNode>,
        nodes: &mut HashMapNodesComposite,
    ) -> Rc<QuadTreeNode> {
        let data = [nw, ne, sw, se].map(|t| Rc::as_ptr(t) as usize);
        let t = nodes.get(&data);
        if let Some(node) = t {
            node.clone()
        } else {
            let result = Rc::new(QuadTreeNode {
                population: (nw.population + ne.population) + (sw.population + se.population),
                data: NodeData::Composite([nw.clone(), ne.clone(), sw.clone(), se.clone()]),
                next: RefCell::new(None),
            });
            nodes.insert(data, result.clone());
            result
        }
    }

    fn split_node(node: &QuadTreeNode) -> [Rc<QuadTreeNode>; 4] {
        match &node.data {
            NodeData::Composite(nodes) => nodes.clone(),
            NodeData::Leaf(_) => panic!("Base node cannot be split"),
        }
    }

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

    fn base_update(&mut self, node: &Rc<QuadTreeNode>) -> Rc<QuadTreeNode> {
        const W: usize = BASE_SIZE / CELLS_IN_CHUNK;
        const H: usize = BASE_SIZE;

        let [nw, ne, sw, se] = Self::split_node(node);
        let (v0, v1, v2, v3) = if let (
            NodeData::Leaf(v0),
            NodeData::Leaf(v1),
            NodeData::Leaf(v2),
            NodeData::Leaf(v3),
        ) = (&nw.data, &ne.data, &sw.data, &se.data)
        {
            (v0, v1, v2, v3)
        } else {
            unreachable!()
        };

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
        Self::get_leaf_node(result, &mut self.nodes_leaf)
    }

    fn update_composite_sequential(
        &mut self,
        node: &Rc<QuadTreeNode>,
        mut size_log2: u32,
    ) -> Rc<QuadTreeNode> {
        let [nw, ne, sw, se] = Self::split_node(node);
        let [_, ne0, sw0, se0] = Self::split_node(&nw);
        let [nw1, _, sw1, se1] = Self::split_node(&ne);
        let [nw2, ne2, _, se2] = Self::split_node(&sw);
        let [nw3, ne3, sw3, _] = Self::split_node(&se);

        size_log2 -= 1;
        let p0 = self.update_node(&nw, size_log2);
        let temp = Self::get_composite_node(&ne0, &nw1, &se0, &sw1, &mut self.nodes_composite);
        let p1 = self.update_node(&temp, size_log2);
        let p2 = self.update_node(&ne, size_log2);
        let temp = Self::get_composite_node(&sw0, &se0, &nw2, &ne2, &mut self.nodes_composite);
        let p3 = self.update_node(&temp, size_log2);
        let temp = Self::get_composite_node(&se0, &sw1, &ne2, &nw3, &mut self.nodes_composite);
        let p4 = self.update_node(&temp, size_log2);
        let temp = Self::get_composite_node(&sw1, &se1, &nw3, &ne3, &mut self.nodes_composite);
        let p5 = self.update_node(&temp, size_log2);
        let p6 = self.update_node(&sw, size_log2);
        let temp = Self::get_composite_node(&ne2, &nw3, &se2, &sw3, &mut self.nodes_composite);
        let p7 = self.update_node(&temp, size_log2);
        let p8 = self.update_node(&se, size_log2);

        let temp = Self::get_composite_node(&p0, &p1, &p3, &p4, &mut self.nodes_composite);
        let q0 = self.update_node(&temp, size_log2);
        let temp = Self::get_composite_node(&p1, &p2, &p4, &p5, &mut self.nodes_composite);
        let q1 = self.update_node(&temp, size_log2);
        let temp = Self::get_composite_node(&p3, &p4, &p6, &p7, &mut self.nodes_composite);
        let q2 = self.update_node(&temp, size_log2);
        let temp = Self::get_composite_node(&p4, &p5, &p7, &p8, &mut self.nodes_composite);
        let q3 = self.update_node(&temp, size_log2);
        Self::get_composite_node(&q0, &q1, &q2, &q3, &mut self.nodes_composite)
    }

    fn update_node(&mut self, node: &Rc<QuadTreeNode>, curr_size_log2: u32) -> Rc<QuadTreeNode> {
        if let Some(node_next) = &*node.next.borrow() {
            return node_next.clone();
        }
        let next = if curr_size_log2 == BASE_SIZE.ilog2() + 1 {
            self.base_update(node)
        } else {
            self.update_composite_sequential(node, curr_size_log2)
        };
        // let t = node.next.get_or_insert_with(|| next);
        *node.next.borrow_mut() = Some(next.clone());
        next
    }

    /// Fills the texture of given resolution with a part of field.
    ///
    /// `viewport_x`, `viewport_y` are reduced to divide by `step`;
    ///
    /// `size` is increased to the next power of two;
    ///
    /// `resolution` is reduced to previous power of two (to fit the texture into `dst`),
    /// doesn't exceed `size`;
    ///
    /// `dst` - buffer of texture.
    pub fn fill_texture(
        &self,
        viewport_x: &mut usize,
        viewport_y: &mut usize,
        size: &mut usize,
        resolution: &mut usize,
        dst: &mut Vec<f64>,
    ) {
        struct ConstArgs {
            viewport_x: usize,
            viewport_y: usize,
            resolution: usize,
            viewport_size: usize,
            step_log2: usize,
        }
        fn inner(
            node: &Rc<QuadTreeNode>,
            curr_x: usize,
            curr_y: usize,
            curr_size_log2: usize,
            const_args: &ConstArgs,
            dst: &mut Vec<f64>,
        ) {
            if const_args.step_log2 == curr_size_log2 {
                let j = (curr_x - const_args.viewport_x) >> const_args.step_log2;
                let i = (curr_y - const_args.viewport_y) >> const_args.step_log2;
                dst[j + i * const_args.resolution] = node.population;
                return;
            }
            match &node.data {
                NodeData::Composite(nodes) => {
                    let half = 1 << (curr_size_log2 - 1);
                    for (i, child) in nodes.iter().enumerate() {
                        let x = curr_x + half * (i & 1 != 0) as usize;
                        let y = curr_y + half * (i & 2 != 0) as usize;
                        if x + half > const_args.viewport_x
                            && x < const_args.viewport_x + const_args.viewport_size
                            && y + half > const_args.viewport_y
                            && y < const_args.viewport_y + const_args.viewport_size
                        {
                            inner(child, x, y, curr_size_log2 - 1, const_args, dst);
                        }
                    }
                }
                NodeData::Leaf(data) => {
                    let k = BASE_SIZE >> const_args.step_log2;
                    let step = 1 << const_args.step_log2;
                    for sy in 0..k {
                        for sx in 0..k {
                            let mut sum = 0;
                            for dy in 0..step {
                                for dx in 0..step {
                                    let x = (sx * step + dx) & (BASE_SIZE - 1);
                                    let y = (sy * step + dy) & (BASE_SIZE - 1);
                                    let pos = (x + y * BASE_SIZE) / CELLS_IN_CHUNK;
                                    let offset = (x + y * BASE_SIZE) % CELLS_IN_CHUNK;
                                    sum += data[pos] >> offset & 1;
                                }
                            }
                            let j = sx + ((curr_x - const_args.viewport_x) >> const_args.step_log2);
                            let i = sy + ((curr_y - const_args.viewport_y) >> const_args.step_log2);
                            dst[j + i * const_args.resolution] = sum as f64;
                        }
                    }
                }
            }
        }

        let step_log2 = (*size / *resolution).max(1).ilog2() as usize;
        println!("STEP={}", 1u64 << step_log2);
        let com_mul = BASE_SIZE.max(1 << step_log2);
        *size = size.next_multiple_of(com_mul) + com_mul;
        *viewport_x = (*viewport_x + 1).next_multiple_of(com_mul) - com_mul;
        *viewport_y = (*viewport_y + 1).next_multiple_of(com_mul) - com_mul;
        *resolution = *size >> step_log2;
        dst.fill(0.);
        dst.resize(*resolution * *resolution, 0.);
        inner(
            &self.root,
            0,
            0,
            self.size.ilog2() as usize,
            &ConstArgs {
                viewport_x: *viewport_x,
                viewport_y: *viewport_y,
                resolution: *resolution,
                viewport_size: *size,
                step_log2,
            },
            dst,
        );
    }

    pub fn blank(size_log2: u32) -> Self {
        assert!(is_x86_feature_detected!("avx2"));
        assert!(is_x86_feature_detected!("popcnt"));
        assert!(size_log2 > BASE_SIZE.ilog2());
        let mut nodes_composite = HashMapNodesComposite::default();
        let mut nodes_leaf = HashMapNodesLeaf::default();
        let mut node = { Self::get_leaf_node([0; CHUNKS_IN_LEAF], &mut nodes_leaf) };
        for _ in BASE_SIZE.ilog2()..size_log2 {
            node = Self::get_composite_node(&node, &node, &node, &node, &mut nodes_composite);
        }
        Self {
            root: node,
            size: 1 << size_log2,
            nodes_composite,
            nodes_leaf,
        }
    }

    pub fn side_length(&self) -> usize {
        self.size
    }

    pub fn get_cell(&self, mut x: usize, mut y: usize) -> bool {
        let mut node = &self.root;
        let mut size = self.size;
        while size >= BASE_SIZE {
            size >>= 1;
            let idx: usize = (x >= size) as usize + 2 * (y >= size) as usize;
            match &node.data {
                NodeData::Composite(nodes) => node = &nodes[idx],
                NodeData::Leaf(data) => {
                    let pos = (x + y * BASE_SIZE) / CELLS_IN_CHUNK;
                    let offset = (x + y * BASE_SIZE) % CELLS_IN_CHUNK;
                    return data[pos] >> offset & 1 != 0;
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
            engine: &mut ConwayFieldHash256,
        ) -> Rc<QuadTreeNode> {
            size >>= 1;
            let idx: usize = (x >= size) as usize + 2 * (y >= size) as usize;
            match &node.data {
                NodeData::Composite(nodes) => {
                    let mut nodes = nodes.clone();
                    x -= (x >= size) as usize * size;
                    y -= (y >= size) as usize * size;
                    nodes[idx] = inner(x, y, size, &nodes[idx], state, engine);
                    let [nw, ne, sw, se] = nodes;
                    ConwayFieldHash256::get_composite_node(
                        &nw,
                        &ne,
                        &sw,
                        &se,
                        &mut engine.nodes_composite,
                    )
                }
                NodeData::Leaf(data) => {
                    let mut data_new: [Chunk; CHUNKS_IN_LEAF] = *data.as_ref();
                    let pos = (x + y * BASE_SIZE) / CELLS_IN_CHUNK;
                    let mask = 1 << ((x + y * BASE_SIZE) % CELLS_IN_CHUNK);
                    if state {
                        data_new[pos] |= mask;
                    } else {
                        data_new[pos] &= !mask;
                    }
                    ConwayFieldHash256::get_leaf_node(data_new, &mut engine.nodes_leaf)
                }
            }
        }

        self.root = inner(x, y, self.size, &self.root.clone(), state, self);
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
            let size_log2 = self.size.ilog2();
            let q = {
                let temp = Self::get_composite_node(top, top, top, top, &mut self.nodes_composite);
                self.update_node(&temp, size_log2 + 1)
            };
            let [se, sw, ne, nw] = Self::split_node(&q);
            self.root = Self::get_composite_node(&nw, &ne, &sw, &se, &mut self.nodes_composite);
        }
    }

    /// Recursively builds OTCA megapixels `depth` times, uses `top_pattern` as the top level.
    ///
    /// If `depth` == 0, every cell is a regular cell, if 1 it is
    /// an OTCA build from regular cells and so on.
    ///
    /// `top_pattern` must consist of zeros and ones.
    pub fn from_recursive_otca_megapixel<const N: usize>(
        depth: u32,
        top_pattern: [[u8; N]; N],
    ) -> Self {
        assert!(N.is_power_of_two());

        const OTCA_SIZE: usize = 2048;
        // dead and alive
        let otca_patterns = ["res/otca_0.rle", "res/otca_1.rle"].map(|path| {
            use std::fs::File;
            use std::io::Read;
            let mut buf = vec![];
            File::open(path).unwrap().read_to_end(&mut buf).unwrap();
            let (w, h, data) = super::parsing_rle::parse_rle(&buf);
            assert_eq!(w, OTCA_SIZE);
            assert_eq!(h, OTCA_SIZE);
            data
        });

        if depth == 0 {
            // TODO
            unimplemented!()
        }

        let mut nodes_composite = HashMapNodesComposite::default();
        let mut nodes_leaf = HashMapNodesLeaf::default();
        let (mut nodes_curr, mut nodes_next) = (vec![], vec![]);
        // creating first-level OTCA nodes
        let mut otca_nodes = [0, 1].map(|i| {
            for y in 0..OTCA_SIZE / BASE_SIZE {
                for x in 0..OTCA_SIZE / BASE_SIZE {
                    let mut data = [0; CHUNKS_IN_LEAF];
                    for sy in 0..BASE_SIZE {
                        for sx in 0..BASE_SIZE {
                            let pos = (sx + sy * BASE_SIZE) / CELLS_IN_CHUNK;
                            let mask = 1 << ((sx + sy * BASE_SIZE) % CELLS_IN_CHUNK);
                            if otca_patterns[i]
                                [(sx + x * BASE_SIZE) + (sy + y * BASE_SIZE) * OTCA_SIZE]
                            {
                                data[pos] |= mask;
                            }
                        }
                    }
                    nodes_curr.push(Self::get_leaf_node(data, &mut nodes_leaf));
                }
            }
            let mut t = OTCA_SIZE / BASE_SIZE;
            while t != 1 {
                for y in (0..t).step_by(2) {
                    for x in (0..t).step_by(2) {
                        let nw = &nodes_curr[x + y * t];
                        let ne = &nodes_curr[(x + 1) + y * t];
                        let sw = &nodes_curr[x + (y + 1) * t];
                        let se = &nodes_curr[(x + 1) + (y + 1) * t];
                        nodes_next.push(Self::get_composite_node(
                            nw,
                            ne,
                            sw,
                            se,
                            &mut nodes_composite,
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
        for _ in 1..depth {
            let otca_nodes_next = [0, 1].map(|i| {
                for y in 0..OTCA_SIZE {
                    for x in 0..OTCA_SIZE {
                        let state = otca_patterns[i][x + y * OTCA_SIZE] as usize;
                        nodes_curr.push(otca_nodes[state].clone());
                    }
                }
                let mut t = OTCA_SIZE;
                while t != 1 {
                    for y in (0..t).step_by(2) {
                        for x in (0..t).step_by(2) {
                            let nw = &nodes_curr[x + y * t];
                            let ne = &nodes_curr[(x + 1) + y * t];
                            let sw = &nodes_curr[x + (y + 1) * t];
                            let se = &nodes_curr[(x + 1) + (y + 1) * t];
                            nodes_next.push(Self::get_composite_node(
                                nw,
                                ne,
                                sw,
                                se,
                                &mut nodes_composite,
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
        assert!(N.is_power_of_two());
        for row in top_pattern {
            for state in row {
                let state = state as usize;
                assert!(state == 0 || state == 1);
                nodes_curr.push(otca_nodes[state].clone());
            }
        }
        let mut t = N;
        while t != 1 {
            for y in (0..t).step_by(2) {
                for x in (0..t).step_by(2) {
                    let nw = &nodes_curr[x + y * t];
                    let ne = &nodes_curr[(x + 1) + y * t];
                    let sw = &nodes_curr[x + (y + 1) * t];
                    let se = &nodes_curr[(x + 1) + (y + 1) * t];
                    nodes_next.push(Self::get_composite_node(
                        nw,
                        ne,
                        sw,
                        se,
                        &mut nodes_composite,
                    ));
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
            size: OTCA_SIZE.pow(depth) * N,
            nodes_composite,
            nodes_leaf,
        }
    }

    pub fn into_mc<P: AsRef<Path>>(&self, path: P) {
        use std::collections::HashMap;
        use std::fs::File;
        use std::io::Write;

        fn inner(
            node: &Rc<QuadTreeNode>,
            size_log2: u32,
            codes: &mut HashMap<*const QuadTreeNode, usize>,
            result: &mut Vec<String>,
        ) {
            if codes.contains_key(&Rc::as_ptr(node)) {
                return;
            }
            let mut s = String::new();
            match &node.data {
                NodeData::Composite(data) => {
                    for t in data.iter() {
                        inner(t, size_log2 - 1, codes, result);
                    }
                    s = format!(
                        "{} {} {} {} {}",
                        size_log2,
                        codes.get(&Rc::as_ptr(&data[0])).unwrap(),
                        codes.get(&Rc::as_ptr(&data[1])).unwrap(),
                        codes.get(&Rc::as_ptr(&data[2])).unwrap(),
                        codes.get(&Rc::as_ptr(&data[3])).unwrap(),
                    );
                }
                NodeData::Leaf(data) => {
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
                }
            }
            let v = if node.population != 0.0 {
                result.push(s);
                result.len()
            } else {
                0
            };
            codes
                .entry(Rc::as_ptr(node) as *const QuadTreeNode)
                .or_insert(v);
        }

        let mut codes = HashMap::new();
        let mut result = vec![];
        inner(&self.root, self.size.ilog2(), &mut codes, &mut result);

        let mut file = File::create(path).unwrap();
        write!(file, "[M2] (hi)\n#R B3/S23\n").unwrap();
        for s in result {
            writeln!(file, "{}", s).unwrap();
        }
    }
}
