/// Returns `size_log2` and row-major vector filled with cells of the parsed RLE pattern.
pub fn parse_rle(data: &[u8]) -> (u32, Vec<u64>) {
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
    let size_log2 = {
        let width = parse_next_number(&mut i);
        let height = parse_next_number(&mut i);
        width
            .max(height)
            .next_power_of_two()
            .ilog2()
            .max(crate::MIN_SIDE_LOG2)
    };
    let n = 1 << size_log2;
    let mut result = vec![0; n * n / 64];
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
                    result[(x + y * n) / 64] |= 1 << (x % 64);
                    x += 1;
                }
                (i, cnt) = (i + 1, 1);
                assert!(x <= n);
            }
            b'b' => {
                (x, i, cnt) = (x + cnt, i + 1, 1);
                assert!(x <= n);
            }
            b'$' => {
                (x, y, i, cnt) = (0, y + cnt, i + 1, 1);
                assert!(y <= n);
            }
            b'!' => {
                y += 1;
                assert!(y <= n);
                break;
            }
            _ => panic!("Unexpected symbol"),
        };
    }
    assert!(y <= n);
    (size_log2, result)
}
