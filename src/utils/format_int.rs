pub fn with_delimiters(value: i128) -> String {
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
