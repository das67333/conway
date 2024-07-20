#[derive(PartialEq, Eq)]
pub enum BrightnessStrategy {
    Linear,
    Golly,
    Custom,
}

impl BrightnessStrategy {
    /// Transforms populations into single-channel image data.
    pub fn transform(&self, resolution: usize, data: &[f64]) -> Vec<u8> {
        assert_eq!(data.len(), resolution * resolution);
        match self {
            Self::Linear => {
                let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let min = data.iter().copied().fold(f64::INFINITY, f64::min);
                data.iter()
                    .map(|x| ((x - min) / (max - min) * u8::MAX as f64) as u8)
                    .collect::<Vec<_>>()
            }
            Self::Golly => data
                .iter()
                .map(|&x| if x != 0.0 { u8::MAX } else { 0 })
                .collect::<Vec<_>>(),
            Self::Custom => {
                let u = data
                    .iter()
                    .cloned()
                    .filter(|&x| x != 0.0)
                    .collect::<Vec<_>>();
                if u.iter().all(|&x| x == u[0]) {
                    let c = if *u.first().unwrap_or(&0.0) != 0.0 {
                        u8::MAX
                    } else {
                        0
                    };
                    return vec![c; resolution * resolution];
                }
                let m = u.iter().sum::<f64>() / u.len() as f64;
                let dev = (u.iter().map(|&x| (x - m) * (x - m)).sum::<f64>()
                    / (u.len() - 1) as f64)
                    .sqrt();
                data.iter()
                    .map(|&x| (((x - m + dev * 0.5) / dev).clamp(0., 1.) * u8::MAX as f64) as u8)
                    .collect::<Vec<_>>()
            }
        }
    }
}
