// Using Kmaps generated at https://charlie-coleman.com/experiments/kmap/
// Slower than a table, and probably more work in the end, but more fun.

use lazy_static::lazy_static;

use super::byte_kmap::ByteKmap;

lazy_static! {
    static ref INVALID_KMAP: ByteKmap =
        ByteKmap::parse(&"abc'df'gh + abdefg'h + abcd'f'gh + abcfg'h' + abcefg'");
    static ref THREE_KMAP: ByteKmap =
        ByteKmap::parse(&"a'b'e'f'g'h + a'b'c'd'ef'g'h' + abfg'h' + abefg' + abef'gh' + abc'e'f'g");
    static ref TWO_KMAP: ByteKmap =
        ByteKmap::parse(&"a'b'fgh' + abch' + a'b'df'g'h' + a'b'cf'g'h' + abgh' + abd'f'g");
}

pub fn bytes_required(b: u8) -> u8 {
    if INVALID_KMAP.test(b) {
        return 0;
    }
    if THREE_KMAP.test(b) {
        return 3;
    }
    if TWO_KMAP.test(b) {
        return 2;
    }
    1
}

#[cfg(test)]
mod test {
    use super::bytes_required;

    const INVALIDS: [usize; 11] = [211, 219, 221, 227, 228, 235, 236, 237, 244, 252, 253];
    const THREES: [usize; 17] = [
        1, 17, 33, 49, 8, 194, 210, 195, 196, 212, 202, 218, 234, 250, 204, 220, 205,
    ];
    const TWOS: [usize; 29] = [
        6, 14, 16, 22, 24, 30, 32, 38, 40, 46, 48, 54, 56, 62, 198, 203, 206, 214, 222, 224, 226,
        230, 232, 238, 240, 242, 246, 248, 254,
    ];

    #[test]
    fn correct_counts() {
        let mut vis = [false; 256];
        for i in INVALIDS.iter() {
            assert_eq!(vis[*i], false);
            assert_eq!(bytes_required(*i as _), 0);
            vis[*i] = true;
        }
        for i in THREES.iter() {
            assert_eq!(vis[*i], false);
            assert_eq!(bytes_required(*i as _), 3);
            vis[*i] = true;
        }
        for i in TWOS.iter() {
            assert_eq!(vis[*i], false);
            assert_eq!(bytes_required(*i as _), 2);
            vis[*i] = true;
        }
        for i in 0..256 {
            if vis[i] {
                continue;
            }
            assert_eq!(bytes_required(i as _), 1);
        }
    }
}
