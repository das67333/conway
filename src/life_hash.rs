use crate::trait_grid::Grid;
use std::rc::Rc;
type HashMap = std::collections::HashMap<u64, Rc<Field>, nohash_hasher::BuildNoHashHasher<u64>>;

#[derive(Debug, Clone)]
struct Field {
    hash: u64,
    data: Option<[Rc<Field>; 4]>,
}

pub struct ConwayField {
    top_field: Rc<Field>,
    size: usize,
    next_centers: HashMap,
    base_fields: [Rc<Field>; 16],
}

impl ConwayField {
    const fn rehash(nw: u64, ne: u64, sw: u64, se: u64) -> u64 {
        const fn xorshift64star(mut x: u64) -> u64 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        }

        let mut x = xorshift64star(nw);
        x ^= xorshift64star(ne)
            .wrapping_add(0x9e3779b9)
            .wrapping_add(x << 6)
            .wrapping_add(x >> 2);
        x ^= xorshift64star(sw)
            .wrapping_add(0x9e3779b9)
            .wrapping_add(x << 6)
            .wrapping_add(x >> 2);
        x ^= xorshift64star(se)
            .wrapping_add(0x9e3779b9)
            .wrapping_add(x << 6)
            .wrapping_add(x >> 2);
        x
    }

    fn build_blank_field(base_fields: &[Rc<Field>; 16], size: usize) -> Rc<Field> {
        if size == 2 {
            base_fields[0].clone()
        } else {
            let p = Self::build_blank_field(base_fields, size / 2);
            Rc::new(Field {
                hash: Self::rehash(p.hash, p.hash, p.hash, p.hash),
                data: Some([0; 4].map(|_| p.clone())),
            })
        }
    }

    fn build_random_field(
        base_fields: &[Rc<Field>; 16],
        size: usize,
        x: usize,
        y: usize,
        values: &Vec<Vec<bool>>,
    ) -> Rc<Field> {
        if size == 2 {
            let i = (values[y][x] as usize)
                + ((values[y][x + 1] as usize) << 1)
                + ((values[y + 1][x] as usize) << 2)
                + ((values[y + 1][x + 1] as usize) << 3);
            base_fields[i].clone()
        } else {
            let half = size / 2;
            let nw = Self::build_random_field(base_fields, half, x, y, values);
            let ne = Self::build_random_field(base_fields, half, x + half, y, values);
            let sw = Self::build_random_field(base_fields, half, x, y + half, values);
            let se = Self::build_random_field(base_fields, half, x + half, y + half, values);
            Rc::new(Field {
                hash: Self::rehash(nw.hash, ne.hash, sw.hash, se.hash),
                data: Some([nw, ne, sw, se]),
            })
        }
    }

    fn unite(&mut self, nw: &Rc<Field>, ne: &Rc<Field>, sw: &Rc<Field>, se: &Rc<Field>) -> Field {
        Field {
            hash: Self::rehash(nw.hash, ne.hash, sw.hash, se.hash),
            data: Some([nw.clone(), ne.clone(), sw.clone(), se.clone()]),
        }
    }

    fn split(field: &Field) -> [Rc<Field>; 4] {
        let [nw, ne, sw, se] = &field.data.as_ref().expect("Trying to split base field");
        [nw.clone(), ne.clone(), sw.clone(), se.clone()]
    }

    fn next_center(&mut self, field: &Field) -> Rc<Field> {
        if let Some(x) = self.next_centers.get(&field.hash) {
            return x.clone();
        }
        let [nw, ne, sw, se] = Self::split(field);
        let [_, ne0, sw0, se0] = Self::split(&nw);
        let [nw1, _, sw1, se1] = Self::split(&ne);
        let [nw2, ne2, _, se2] = Self::split(&sw);
        let [nw3, ne3, sw3, _] = Self::split(&se);

        let u1 = self.unite(&ne0, &nw1, &se0, &sw1);
        let u3 = self.unite(&sw0, &se0, &nw2, &ne2);
        let u4 = self.unite(&se0, &sw1, &ne2, &nw3);
        let u5 = self.unite(&sw1, &se1, &nw3, &ne3);
        let u7 = self.unite(&ne2, &nw3, &se2, &sw3);

        let p0 = self.next_center(&nw);
        let p1 = self.next_center(&u1);
        let p2 = self.next_center(&ne);
        let p3 = self.next_center(&u3);
        let p4 = self.next_center(&u4);
        let p5 = self.next_center(&u5);
        let p6 = self.next_center(&sw);
        let p7 = self.next_center(&u7);
        let p8 = self.next_center(&se);

        let w0 = self.unite(&p0, &p1, &p3, &p4);
        let w1 = self.unite(&p1, &p2, &p4, &p5);
        let w2 = self.unite(&p3, &p4, &p6, &p7);
        let w3 = self.unite(&p4, &p5, &p7, &p8);

        let q0 = self.next_center(&w0);
        let q1 = self.next_center(&w1);
        let q2 = self.next_center(&w2);
        let q3 = self.next_center(&w3);
        let result = Rc::new(self.unite(&q0, &q1, &q2, &q3));
        self.next_centers.insert(field.hash, result.clone());
        result
    }

    fn insert_base_fields(next_centers: &mut HashMap) -> [Rc<Field>; 16] {
        let data2x2 = (0..16)
            .map(|i: u8| [i & 1, (i >> 1) & 1, (i >> 2) & 1, (i >> 3) & 1])
            .collect::<Vec<_>>();
        let fields2x2 = (0..16)
            .map(|i| {
                Rc::new(Field {
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
                        let mut next_center_idx = 0;
                        for (i, &val) in [dnw[3], dne[2], dsw[1], dse[0]].iter().enumerate() {
                            let state = if val != 0 {
                                neibs[i] == 2 || neibs[i] == 3
                            } else {
                                neibs[i] == 3
                            };
                            if state {
                                next_center_idx |= 1 << i;
                            }
                        }
                        next_centers.insert(hash, fields2x2[next_center_idx].clone());
                    }
                }
            }
        }
        assert_eq!(next_centers.len(), 1 << 16, "Bad hash function");
        fields2x2.try_into().unwrap()
    }

    fn update_inner(&mut self) {
        let top = &Rc::new(self.top_field.clone());
        let p = self.unite(top, top, top, top);
        let q = self.next_center(&p);
        let [se, sw, ne, nw] = Self::split(&q);
        self.top_field = Rc::new(self.unite(&nw, &ne, &sw, &se));
    }
}

impl Grid for ConwayField {
    fn blank(width: usize, height: usize) -> Self {
        assert_eq!(width, height);
        assert!(width.is_power_of_two());
        assert!(width >= 4);
        let mut next_centers: HashMap = HashMap::default();
        let base_fields = Self::insert_base_fields(&mut next_centers);
        let top_field = Self::build_blank_field(&base_fields, width);
        Self {
            top_field,
            size: width,
            next_centers,
            base_fields,
        }
    }

    fn random(width: usize, height: usize, seed: Option<u64>, fill_rate: f64) -> Self {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;

        let mut rng = if let Some(x) = seed {
            ChaCha8Rng::seed_from_u64(x)
        } else {
            ChaCha8Rng::from_entropy()
        };
        assert_eq!(width, height);
        assert!(width.is_power_of_two());
        assert!(width >= 4);

        let values = (0..width)
            .map(|_| {
                (0..width)
                    .map(|_| rng.gen_bool(fill_rate))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut next_centers = HashMap::default();
        let base_fields = Self::insert_base_fields(&mut next_centers);
        let top_field = Self::build_random_field(&base_fields, width, 0, 0, &values);
        Self {
            top_field,
            size: width,
            next_centers,
            base_fields,
        }
    }

    fn size(&self) -> (usize, usize) {
        (self.size, self.size)
    }

    fn get(&self, x: usize, y: usize) -> bool {
        fn get_inner(field: &Field, size: usize, mut x: usize, mut y: usize) -> bool {
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
                get_inner(&fields[idx], half, x, y)
            } else {
                (field.hash >> idx) & 1 != 0
            }
        }

        get_inner(&self.top_field, self.size, x, y)
    }

    fn set(&mut self, x: usize, y: usize, value: bool) {
        fn set_inner(
            base_fields: &[Rc<Field>; 16],
            field: &Field,
            size: usize,
            mut x: usize,
            mut y: usize,
            value: bool,
        ) -> Rc<Field> {
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
                let new_field = set_inner(base_fields, &fields[idx], half, x, y, value);
                let mut fields = fields.clone();
                fields[idx] = new_field;
                let hash = ConwayField::rehash(
                    fields[0].hash,
                    fields[1].hash,
                    fields[2].hash,
                    fields[3].hash,
                );
                Rc::new(Field {
                    data: Some(fields),
                    hash,
                })
            } else {
                let mut fields = field.hash;
                let mask = 1 << idx;
                if value {
                    fields |= mask;
                } else {
                    fields &= !mask;
                }
                base_fields[fields as usize].clone()
            }
        }

        self.top_field = set_inner(
            &self.base_fields,
            &self.top_field,
            self.size,
            x,
            y,
            value,
        );
    }

    fn update(&mut self, n: usize) {
        let m = self.size / 2;
        assert!(n % m == 0);
        for _ in 0..n / m {
            self.update_inner();
        }
    }

    fn draw(&self, screen: &mut [u8]) {
        fn draw_inner(field: &Field, size: usize, x: usize, y: usize, values: &mut Vec<Vec<bool>>) {
            if size == 2 {
                values[y][x] = field.hash & 1 != 0;
                values[y][x + 1] = (field.hash >> 1) & 1 != 0;
                values[y + 1][x] = (field.hash >> 2) & 1 != 0;
                values[y + 1][x + 1] = (field.hash >> 3) & 1 != 0;
            } else {
                let half = size / 2;
                draw_inner(&field.data.as_ref().unwrap()[0], half, x, y, values);
                draw_inner(&field.data.as_ref().unwrap()[1], half, x + half, y, values);
                draw_inner(&field.data.as_ref().unwrap()[2], half, x, y + half, values);
                draw_inner(
                    &field.data.as_ref().unwrap()[3],
                    half,
                    x + half,
                    y + half,
                    values,
                );
            }
        }

        const BYTES_IN_PIXEL: usize = 4;
        assert_eq!(screen.len(), BYTES_IN_PIXEL * self.size * self.size);
        let mut values = (0..self.size)
            .map(|_| vec![false; self.size])
            .collect::<Vec<_>>();
        draw_inner(&self.top_field, self.size, 0, 0, &mut values);
        for (pixel, &value) in screen
            .chunks_exact_mut(BYTES_IN_PIXEL)
            .zip(values.iter().flatten())
        {
            pixel.copy_from_slice(&if value {
                [0, 0xff, 0xff, 0xff]
            } else {
                [0, 0, 0, 0xff]
            });
        }
    }
}
