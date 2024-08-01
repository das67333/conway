pub struct NiceInt(i128);

impl NiceInt {
    pub fn from(value: impl Into<i128>) -> Self {
        Self(value.into())
    }

    pub fn from_usize(value: usize) -> Self {
        Self(value as i128)
    }

    pub fn from_f64(value: f64) -> Self {
        Self(value as i128)
    }
}

impl std::fmt::Display for NiceInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", with_delimiters(self.0))
    }
}

fn with_delimiters(value: i128) -> String {
    let mut result = value
        .abs()
        .to_string()
        .chars()
        .rev()
        .collect::<Vec<char>>()
        .chunks(3)
        .map(|c| c.iter().rev().collect::<String>())
        .rev()
        .collect::<Vec<String>>()
        .join("'");
    if value < 0 {
        result.insert(0, '-');
    }
    result
}
