pub struct ByteKmap {
    matchings: Vec<(u8, u8)>,
}

impl ByteKmap {
    /// Parses Kmaps generated at https://charlie-coleman.com/experiments/kmap/
    pub fn parse(s: &str) -> Self {
        // Assumes a is bit n-1, and counts down
        let bits = 8;
        let parse_product = |prod: &str| {
            let mut neg = false;
            let mut mask = 0;
            let mut value = 0;

            for c in prod.chars().rev() {
                if c == '\'' {
                    assert!(!neg, "Two negations in a row");
                    neg = true;
                } else {
                    let ascii = c as u8;
                    let index = bits - (ascii - 96);
                    assert!(index < 26, "Character out of range");
                    mask |= 1 << index;
                    if !neg {
                        value |= 1 << index;
                    } else {
                        neg = false;
                    }
                }
            }
            assert!(!neg, "Unused negation");
            (mask, value)
        };

        ByteKmap {
            matchings: s.split(" + ").map(parse_product).collect(),
        }
    }

    /// Create a trivial kmap consisting of a single target byte
    pub fn single(val: u8) -> Self {
        ByteKmap {
            matchings: vec![(0xff, val)],
        }
    }

    /// Determine if the given byte matches the kmap
    pub fn test(&self, b: u8) -> bool {
        self.matchings
            .iter()
            .any(|(mask, value)| (b & mask) == *value)
    }

    /// Generate a list of all values that match the pattern
    pub fn enumerate(&self) -> Vec<u8> {
        let enummatching = |&(mask, value): &(u8, u8)| {
            let mut res = Vec::new();

            let mut i: u8 = 0;
            loop {
                res.push(i | value);

                i = ((i | mask).wrapping_add(1)) & !mask;

                if i == 0 {
                    return res;
                }
            }
        };
        self.matchings.iter().flat_map(enummatching).collect()
    }
}

#[cfg(test)]
mod test {
    use super::ByteKmap;

    #[test]
    fn enumerate_test() {
        let map = ByteKmap::parse(&"bc'e'f'h + abcd'efgh");

        let mut values = map.enumerate();
        values.sort_unstable();
        assert_eq!(vec![65, 67, 81, 83, 193, 195, 209, 211, 239], values);

        for i in 0..=255 {
            assert_eq!(values.contains(&i), map.test(i));
        }
    }
}
