use crate::arith::mul_mod;

/// Barrett reducer for products of residues modulo m, with m < 2^32.
#[derive(Clone, Copy)]
pub struct Barrett32 {
    modulus: u64,
    reciprocal: u128,
}

impl Barrett32 {
    pub fn new(modulus: u64) -> Self {
        assert!(modulus > 1 && modulus < (1u64 << 32));
        Self {
            modulus,
            reciprocal: (1u128 << 64) / modulus as u128,
        }
    }

    #[inline(always)]
    pub fn multiply(self, a: u64, b: u64) -> u64 {
        let product = a * b;
        let quotient = ((product as u128 * self.reciprocal) >> 64) as u64;
        let mut remainder = product - quotient * self.modulus;
        if remainder >= self.modulus {
            remainder -= self.modulus;
        }
        remainder
    }
}

/// Montgomery reducer with R=2^64, specialized to odd moduli below 2^32.
#[derive(Clone, Copy)]
pub struct Montgomery32 {
    modulus: u64,
    negative_inverse: u64,
    r2: u64,
}

impl Montgomery32 {
    pub fn new(modulus: u64) -> Self {
        assert!(modulus > 1 && modulus & 1 == 1 && modulus < (1u64 << 32));
        let mut inverse = 1u64;
        for _ in 0..6 {
            inverse = inverse.wrapping_mul(2u64.wrapping_sub(modulus.wrapping_mul(inverse)));
        }
        let r = ((1u128 << 64) % modulus as u128) as u64;
        Self {
            modulus,
            negative_inverse: inverse.wrapping_neg(),
            r2: mul_mod(r, r, modulus),
        }
    }

    #[inline(always)]
    fn reduce(self, product: u128) -> u64 {
        let correction = (product as u64).wrapping_mul(self.negative_inverse);
        let sum = product + correction as u128 * self.modulus as u128;
        let mut result = (sum >> 64) as u64;
        if result >= self.modulus {
            result -= self.modulus;
        }
        result
    }

    #[inline(always)]
    pub fn encode(self, value: u64) -> u64 {
        self.reduce(value as u128 * self.r2 as u128)
    }

    #[inline(always)]
    pub fn multiply(self, a: u64, b: u64) -> u64 {
        self.reduce(a as u128 * b as u128)
    }

    #[inline(always)]
    pub fn decode(self, value: u64) -> u64 {
        self.reduce(value as u128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reducers_match_u128_modulo() {
        for modulus in [3, 103, 65_537, 1_000_003, 3_999_999_959] {
            let barrett = Barrett32::new(modulus);
            let montgomery = Montgomery32::new(modulus);
            let values = [0, 1, 2, 17, modulus / 2, modulus - 1];
            for a in values {
                for b in values {
                    let expected = mul_mod(a, b, modulus);
                    assert_eq!(barrett.multiply(a, b), expected);
                    let got = montgomery
                        .decode(montgomery.multiply(montgomery.encode(a), montgomery.encode(b)));
                    assert_eq!(got, expected);
                }
            }
        }
    }
}
