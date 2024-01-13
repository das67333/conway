#[derive(Default)]
pub struct EmptyHasher {
    n: u128,
}

impl std::hash::Hasher for EmptyHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.n as u64
    }
    #[inline(always)]
    fn write_u128(&mut self, x: u128) {
        self.n = x
    }
    #[inline(always)]
    fn write(&mut self, _input: &[u8]) {
        unimplemented!()
    }
}

pub type EmptyHasherBuilder = std::hash::BuildHasherDefault<EmptyHasher>;
