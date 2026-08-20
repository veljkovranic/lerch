#[inline(always)]
pub fn mul_mod(a: u64, b: u64, modulus: u64) -> u64 {
    ((a as u128 * b as u128) % modulus as u128) as u64
}

pub fn pow_mod(mut base: u64, mut exponent: u64, modulus: u64) -> u64 {
    let mut result = 1 % modulus;
    while exponent != 0 {
        if exponent & 1 != 0 {
            result = mul_mod(result, base, modulus);
        }
        exponent >>= 1;
        if exponent != 0 {
            base = mul_mod(base, base, modulus);
        }
    }
    result
}

pub fn inverse_mod(a: u64, prime: u64) -> u64 {
    debug_assert!(a != 0 && a < prime);
    pow_mod(a, prime - 2, prime)
}

/// Distinct prime divisors. `trial_primes` must include all primes through sqrt(n).
pub fn distinct_prime_factors(mut n: u64, trial_primes: &[u64]) -> Vec<u64> {
    let mut factors = Vec::new();
    for &q in trial_primes {
        if q > n / q {
            break;
        }
        if n % q == 0 {
            factors.push(q);
            while n % q == 0 {
                n /= q;
            }
        }
    }
    if n > 1 {
        factors.push(n);
    }
    factors
}

pub fn primitive_root(p: u64, trial_primes: &[u64]) -> u64 {
    assert!(p >= 3);
    let factors = distinct_prime_factors(p - 1, trial_primes);
    'candidate: for g in 2..p {
        for &q in &factors {
            if pow_mod(g, (p - 1) / q, p) == 1 {
                continue 'candidate;
            }
        }
        return g;
    }
    unreachable!("every prime has a primitive root")
}

pub fn fermat_quotient_mod_p(a: u64, p: u64) -> u64 {
    let p2 = p.checked_mul(p).expect("p^2 must fit u64");
    (pow_mod(a, p - 1, p2) - 1) / p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_generate_the_group() {
        let primes = crate::sieve::simple_primes(100);
        for p in crate::sieve::segmented_primes(3, 100, &primes) {
            let g = primitive_root(p, &primes);
            let mut seen = vec![false; p as usize];
            let mut x = 1;
            for _ in 0..p - 1 {
                assert!(!seen[x as usize]);
                seen[x as usize] = true;
                x = mul_mod(x, g, p);
            }
            assert_eq!(x, 1);
        }
    }
}
