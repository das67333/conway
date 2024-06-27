/// Returns width, height and row-major vector filled with cells of the parsed RLE pattern
pub fn parse_rle(data: &[u8]) -> (u64, u64, Vec<bool>) {
    let parse_next_number = |i: &mut usize| {
        while !data[*i].is_ascii_digit() {
            *i += 1;
        }
        let j = {
            let mut j = *i;
            while j < data.len() && data[j].is_ascii_digit() {
                j += 1;
            }
            j
        };
        let ans = String::from_utf8(data[*i..j].to_vec())
            .unwrap()
            .parse::<usize>()
            .unwrap();
        *i = j;
        ans
    };

    let mut i = 0;
    // skipping comment lines
    while data[i] == b'#' {
        while data[i] != b'\n' {
            i += 1;
        }
        i += 1;
    }
    // next line must start with 'x'; parsing sizes
    let width = parse_next_number(&mut i);
    let height = parse_next_number(&mut i);
    let mut result = vec![false; width * height];
    while data[i] != b'\n' {
        i += 1;
    }
    i += 1;
    // run-length encoded pattern data
    let (mut x, mut y, mut cnt) = (0, 0, 1);
    while i < data.len() {
        match data[i] {
            b'\n' => i += 1,
            b'0'..=b'9' => cnt = parse_next_number(&mut i),
            b'o' => {
                for _ in 0..cnt {
                    result[x + y * width] = true;
                    x += 1;
                }
                (i, cnt) = (i + 1, 1);
                assert!(x <= width);
            }
            b'b' => {
                (x, i, cnt) = (x + cnt, i + 1, 1);
                assert!(x <= width);
            }
            b'$' => {
                (x, y, i, cnt) = (0, y + cnt, i + 1, 1);
                assert!(y <= height);
            }
            b'!' => {
                y += 1;
                assert!(y <= height);
                break;
            }
            _ => panic!("Unexpected symbol"),
        };
    }
    assert!(y <= height);
    (width as u64, height as u64, result)
}
