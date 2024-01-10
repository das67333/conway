use super::hash_256::ConwayFieldHash256;

pub fn paste_rle(life: &mut ConwayFieldHash256, x: usize, y: usize, data: &[u8]) {
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
    while data[i] != b'\n' {
        i += 1;
    }
    i += 1;
    // run-length encoded pattern data
    let (mut dx, mut dy, mut cnt) = (0, 0, 1);
    while i < data.len() {
        let c = data[i];
        match c {
            b'\n' => i += 1,
            b'0'..=b'9' => cnt = parse_next_number(&mut i),
            b'o' => {
                for _ in 0..cnt {
                    life.set_cell(x + dx, y + dy, true);
                    dx += 1
                }
                (i, cnt) = (i + 1, 1);
                assert!(
                    dx <= width,
                    "i={} {:?}",
                    i,
                    String::from_utf8(data[i - 36..=i].to_vec())
                );
            }
            b'b' => {
                (dx, i, cnt) = (dx + cnt, i + 1, 1);
                assert!(dx <= width);
            }
            b'$' => {
                (dx, dy, i, cnt) = (0, dy + cnt, i + 1, 1);
                assert!(dy <= height);
            }
            b'!' => {
                dy += 1;
                assert!(dy <= height);
                break;
            }
            _ => panic!("Unexpected symbol"),
        };
    }
    assert!(dy <= height);
}
