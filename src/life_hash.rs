use std::rc::Rc;
type HashMap =
    std::collections::HashMap<u64, Rc<QuadTreeNode>, nohash_hasher::BuildNoHashHasher<u64>>;

#[derive(Debug, Clone)]
struct QuadTreeNode {
    hash: u64,
    data: Option<[Rc<QuadTreeNode>; 4]>,
}

pub struct ConwayField {
    root: Rc<QuadTreeNode>,
    size: usize,
    node_updates: HashMap,
    base_nodes: [Rc<QuadTreeNode>; 16],
}

impl ConwayField {
    const fn rehash(nw: u64, ne: u64, sw: u64, se: u64) -> u64 {
        const fn xorshift64(mut x: u64) -> u64 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        }
        const fn merge(x: u64, y: u64) -> u64 {
            x ^ y
                .wrapping_add(0x9e3779b9)
                .wrapping_add(x << 6)
                .wrapping_add(x >> 2)
        }
        merge(
            merge(merge(xorshift64(nw), xorshift64(ne)), xorshift64(sw)),
            xorshift64(se),
        )
    }

    fn build_blank_field(base_nodes: &[Rc<QuadTreeNode>; 16], size: usize) -> Rc<QuadTreeNode> {
        if size == 2 {
            base_nodes[0].clone()
        } else {
            let p = Self::build_blank_field(base_nodes, size / 2);
            Rc::new(QuadTreeNode {
                hash: Self::rehash(p.hash, p.hash, p.hash, p.hash),
                data: Some([0; 4].map(|_| p.clone())),
            })
        }
    }

    fn unite_nodes(
        &mut self,
        nw: &Rc<QuadTreeNode>,
        ne: &Rc<QuadTreeNode>,
        sw: &Rc<QuadTreeNode>,
        se: &Rc<QuadTreeNode>,
    ) -> QuadTreeNode {
        QuadTreeNode {
            hash: Self::rehash(nw.hash, ne.hash, sw.hash, se.hash),
            data: Some([nw.clone(), ne.clone(), sw.clone(), se.clone()]),
        }
    }

    fn split_node(node: &QuadTreeNode) -> [Rc<QuadTreeNode>; 4] {
        let [nw, ne, sw, se] = &node.data.as_ref().expect("Trying to split base node");
        [nw.clone(), ne.clone(), sw.clone(), se.clone()]
    }

    fn update_node(&mut self, node: &QuadTreeNode) -> Rc<QuadTreeNode> {
        if let Some(x) = self.node_updates.get(&node.hash) {
            return x.clone();
        }
        let [nw, ne, sw, se] = Self::split_node(node);
        let [_, ne0, sw0, se0] = Self::split_node(&nw);
        let [nw1, _, sw1, se1] = Self::split_node(&ne);
        let [nw2, ne2, _, se2] = Self::split_node(&sw);
        let [nw3, ne3, sw3, _] = Self::split_node(&se);

        let u1 = self.unite_nodes(&ne0, &nw1, &se0, &sw1);
        let u3 = self.unite_nodes(&sw0, &se0, &nw2, &ne2);
        let u4 = self.unite_nodes(&se0, &sw1, &ne2, &nw3);
        let u5 = self.unite_nodes(&sw1, &se1, &nw3, &ne3);
        let u7 = self.unite_nodes(&ne2, &nw3, &se2, &sw3);

        let p0 = self.update_node(&nw);
        let p1 = self.update_node(&u1);
        let p2 = self.update_node(&ne);
        let p3 = self.update_node(&u3);
        let p4 = self.update_node(&u4);
        let p5 = self.update_node(&u5);
        let p6 = self.update_node(&sw);
        let p7 = self.update_node(&u7);
        let p8 = self.update_node(&se);

        let w0 = self.unite_nodes(&p0, &p1, &p3, &p4);
        let w1 = self.unite_nodes(&p1, &p2, &p4, &p5);
        let w2 = self.unite_nodes(&p3, &p4, &p6, &p7);
        let w3 = self.unite_nodes(&p4, &p5, &p7, &p8);

        let q0 = self.update_node(&w0);
        let q1 = self.update_node(&w1);
        let q2 = self.update_node(&w2);
        let q3 = self.update_node(&w3);
        let result = Rc::new(self.unite_nodes(&q0, &q1, &q2, &q3));
        self.node_updates.insert(node.hash, result.clone());
        result
    }

    fn insert_base_nodes(hashmap: &mut HashMap) -> [Rc<QuadTreeNode>; 16] {
        let data2x2 = (0..16)
            .map(|i: u8| [i & 1, i >> 1 & 1, i >> 2 & 1, i >> 3 & 1])
            .collect::<Vec<_>>();
        let fields2x2 = (0..16)
            .map(|i| {
                Rc::new(QuadTreeNode {
                    data: None,
                    hash: i,
                })
            })
            .collect::<Vec<_>>();
        for (inw, dnw) in data2x2.iter().enumerate() {
            for (ine, dne) in data2x2.iter().enumerate() {
                for (isw, dsw) in data2x2.iter().enumerate() {
                    for (ise, dse) in data2x2.iter().enumerate() {
                        let hash = Self::rehash(inw as u64, ine as u64, isw as u64, ise as u64);
                        let neibs = [
                            dnw[0] + dnw[1] + dnw[2] + dne[0] + dne[2] + dsw[0] + dsw[1] + dse[0],
                            dnw[1] + dnw[3] + dne[0] + dne[1] + dne[3] + dsw[1] + dse[0] + dse[1],
                            dnw[2] + dnw[3] + dne[2] + dsw[0] + dsw[2] + dsw[3] + dse[0] + dse[2],
                            dnw[3] + dne[2] + dne[3] + dsw[1] + dsw[3] + dse[1] + dse[2] + dse[3],
                        ];
                        let mut updated_idx = 0;
                        for (i, &val) in [dnw[3], dne[2], dsw[1], dse[0]].iter().enumerate() {
                            let state = if val != 0 {
                                neibs[i] == 2 || neibs[i] == 3
                            } else {
                                neibs[i] == 3
                            };
                            if state {
                                updated_idx |= 1 << i;
                            }
                        }
                        hashmap.insert(hash, fields2x2[updated_idx].clone());
                    }
                }
            }
        }
        assert_eq!(hashmap.len(), 1 << 16, "Bad hash function");
        fields2x2.try_into().unwrap()
    }

    fn update_inner(&mut self) {
        let top = &Rc::new(self.root.clone());
        let p = self.unite_nodes(top, top, top, top);
        let q = self.update_node(&p);
        let [se, sw, ne, nw] = Self::split_node(&q);
        self.root = Rc::new(self.unite_nodes(&nw, &ne, &sw, &se));
    }
}

impl crate::CellularAutomaton for ConwayField {
    fn blank(width: usize, height: usize) -> Self {
        assert_eq!(width, height);
        assert!(width >= 4 && width.is_power_of_two());
        let mut node_updates: HashMap = HashMap::default();
        let base_nodes = Self::insert_base_nodes(&mut node_updates);
        let root = Self::build_blank_field(&base_nodes, width);
        Self {
            root,
            size: width,
            node_updates,
            base_nodes,
        }
    }

    fn get_size(&self) -> (usize, usize) {
        (self.size, self.size)
    }

    fn get_cell(&self, x: usize, y: usize) -> bool {
        fn get_inner(field: &QuadTreeNode, size: usize, mut x: usize, mut y: usize) -> bool {
            let half = size / 2;
            let mut idx: usize = 0;
            if x >= half {
                idx += 1;
                x -= half;
            }
            if y >= half {
                idx += 2;
                y -= half;
            }
            if let Some(fields) = &field.data {
                get_inner(&fields[idx], half, x, y)
            } else {
                field.hash >> idx & 1 != 0
            }
        }

        get_inner(&self.root, self.size, x, y)
    }

    fn get_cells(&self) -> Vec<bool> {
        fn get_inner(
            field: &QuadTreeNode,
            size: usize,
            x: usize,
            y: usize,
            states: &mut Vec<&mut [bool]>,
        ) {
            if let Some(fields) = &field.data {
                let half = size / 2;
                get_inner(&fields[0], half, x, y, states);
                get_inner(&fields[1], half, x + half, y, states);
                get_inner(&fields[2], half, x, y + half, states);
                get_inner(&fields[3], half, x + half, y + half, states);
            } else {
                states[y][x] = field.hash >> 0 & 1 != 0;
                states[y][x + 1] = field.hash >> 1 & 1 != 0;
                states[y + 1][x] = field.hash >> 2 & 1 != 0;
                states[y + 1][x + 1] = field.hash >> 3 & 1 != 0;
            }
        }

        let mut states = vec![false; self.size * self.size];
        {
            let mut states = states.chunks_exact_mut(self.size).collect::<Vec<_>>();
            get_inner(&self.root, self.size, 0, 0, &mut states);
        }
        states
    }

    fn set_cell(&mut self, x: usize, y: usize, state: bool) {
        fn set_inner(
            base_nodes: &[Rc<QuadTreeNode>; 16],
            field: &QuadTreeNode,
            size: usize,
            mut x: usize,
            mut y: usize,
            state: bool,
        ) -> Rc<QuadTreeNode> {
            let half = size / 2;
            let mut idx = 0;
            if x >= half {
                idx += 1;
                x -= half;
            }
            if y >= half {
                idx += 2;
                y -= half;
            }
            if let Some(fields) = &field.data {
                let new_field = set_inner(base_nodes, &fields[idx], half, x, y, state);
                let mut fields = fields.clone();
                fields[idx] = new_field;
                let hash = ConwayField::rehash(
                    fields[0].hash,
                    fields[1].hash,
                    fields[2].hash,
                    fields[3].hash,
                );
                Rc::new(QuadTreeNode {
                    data: Some(fields),
                    hash,
                })
            } else {
                let mut fields = field.hash;
                let mask = 1 << idx;
                if state {
                    fields |= mask;
                } else {
                    fields &= !mask;
                }
                base_nodes[fields as usize].clone()
            }
        }

        self.root = set_inner(&self.base_nodes, &self.root, self.size, x, y, state);
    }

    fn set_cells(&mut self, states: &[bool]) {
        fn set_inner(
            base_nodes: &[Rc<QuadTreeNode>; 16],
            size: usize,
            x: usize,
            y: usize,
            states: &Vec<&[bool]>,
        ) -> Rc<QuadTreeNode> {
            if size == 2 {
                let i = (states[y][x] as usize)
                    + ((states[y][x + 1] as usize) << 1)
                    + ((states[y + 1][x] as usize) << 2)
                    + ((states[y + 1][x + 1] as usize) << 3);
                base_nodes[i].clone()
            } else {
                let half = size / 2;
                let nw = set_inner(base_nodes, half, x, y, states);
                let ne = set_inner(base_nodes, half, x + half, y, states);
                let sw = set_inner(base_nodes, half, x, y + half, states);
                let se = set_inner(base_nodes, half, x + half, y + half, states);
                Rc::new(QuadTreeNode {
                    hash: ConwayField::rehash(nw.hash, ne.hash, sw.hash, se.hash),
                    data: Some([nw, ne, sw, se]),
                })
            }
        }

        assert_eq!(states.len(), self.size * self.size);
        let states = states.chunks_exact(self.size).collect::<Vec<_>>();
        self.root = set_inner(&self.base_nodes, self.size, 0, 0, &states);
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
            self.update_inner();
        }
    }
}
