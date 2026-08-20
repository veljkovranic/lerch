pub fn integer_sqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let mut x = (n as f64).sqrt() as u64;
    while (x + 1) <= n / (x + 1) {
        x += 1;
    }
    while x > n / x {
        x -= 1;
    }
    x
}

pub fn simple_primes(limit: u64) -> Vec<u64> {
    if limit < 2 {
        return Vec::new();
    }
    let mut prime = vec![true; limit as usize + 1];
    prime[0] = false;
    prime[1] = false;
    let mut q = 2usize;
    while q <= limit as usize / q {
        if prime[q] {
            for multiple in (q * q..=limit as usize).step_by(q) {
                prime[multiple] = false;
            }
        }
        q += 1;
    }
    prime
        .into_iter()
        .enumerate()
        .filter_map(|(n, yes)| yes.then_some(n as u64))
        .collect()
}

/// Return all primes in the inclusive interval. Memory use is O(end-start).
pub fn segmented_primes(start: u64, end: u64, base_primes: &[u64]) -> Vec<u64> {
    if start > end || end < 2 {
        return Vec::new();
    }
    let mut result = Vec::new();
    if start <= 2 && end >= 2 {
        result.push(2);
    }
    let first = start.max(3) | 1;
    if first > end {
        return result;
    }
    let count = ((end - first) / 2 + 1) as usize;
    let mut prime = vec![true; count];
    for &q in base_primes {
        if q == 2 {
            continue;
        }
        if q > end / q {
            break;
        }
        let q2 = q * q;
        let mut multiple = first.div_ceil(q) * q;
        if multiple < q2 {
            multiple = q2;
        }
        if multiple & 1 == 0 {
            multiple += q;
        }
        while multiple <= end {
            prime[((multiple - first) / 2) as usize] = false;
            match multiple.checked_add(2 * q) {
                Some(next) => multiple = next,
                None => break,
            }
        }
    }
    result.extend(
        prime
            .into_iter()
            .enumerate()
            .filter_map(|(i, yes)| yes.then_some(first + 2 * i as u64)),
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segmented_matches_simple() {
        let all = simple_primes(10_000);
        let base = simple_primes(100);
        assert_eq!(segmented_primes(0, 10_000, &base), all);
        assert_eq!(
            segmented_primes(1234, 9876, &base),
            all.into_iter()
                .filter(|&p| (1234..=9876).contains(&p))
                .collect::<Vec<_>>()
        );
    }
}
