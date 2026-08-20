use crate::arith::{fermat_quotient_mod_p, inverse_mod, mul_mod, primitive_root};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default)]
pub struct MomentOptions {
    pub q3: bool,
    pub q4: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invariants {
    pub p: u64,
    pub primitive_root: u64,
    pub q1: u64,
    pub q2: u64,
    pub q3: Option<u64>,
    pub q4: Option<u64>,
    pub lerch_remainder: Option<u64>,
    pub generalized_k: Option<u64>,
    pub is_lerch: bool,
    pub is_gy_exceptional: bool,
    pub is_wilson: bool,
    pub q1_equals_2: bool,
}

impl Invariants {
    pub fn rare(&self) -> bool {
        self.is_lerch
            || self.is_gy_exceptional
            || self.is_wilson
            || self.q1_equals_2
            || self.q3 == Some(0)
            || self.q4 == Some(0)
    }
}

/// Primitive-root recurrence. The loop counts u_0 through u_{p-2} exactly once.
pub fn recurrence_invariants(p: u64, trial_primes: &[u64], moments: MomentOptions) -> Invariants {
    if p == 2 {
        return Invariants {
            p,
            primitive_root: 1,
            q1: 0,
            q2: 0,
            q3: moments.q3.then_some(0),
            q4: moments.q4.then_some(0),
            lerch_remainder: None,
            generalized_k: None,
            is_lerch: false,
            is_gy_exceptional: true,
            is_wilson: false,
            q1_equals_2: false,
        };
    }
    assert!(p <= 4_000_000_000, "fixed-width search limit exceeded");
    let g = primitive_root(p, trial_primes);
    let g_inverse = inverse_mod(g, p);
    let qg = fermat_quotient_mod_p(g, p);
    let mut c = 1u64;
    let mut v = 1u64;
    let mut u = 0u64;
    let mut q1 = 0u64;
    let mut q2 = 0u64;
    let mut q3 = 0u64;
    let mut q4 = 0u64;

    for j in 0..p - 1 {
        q1 += u;
        if q1 >= p {
            q1 -= p;
        }
        q2 = (q2 + u * u) % p;
        if moments.q3 || moments.q4 {
            let u2 = mul_mod(u, u, p);
            if moments.q3 {
                q3 = (q3 + mul_mod(u2, u, p)) % p;
            }
            if moments.q4 {
                q4 = (q4 + mul_mod(u2, u2, p)) % p;
            }
        }
        if j + 1 == p - 1 {
            break;
        }
        let product = g * c;
        let k = product / p;
        c = product - k * p;
        v = (v * g_inverse) % p;
        u = (u + qg + k * v) % p;
        debug_assert_eq!(mul_mod(c, v, p), 1);
    }

    let expression = (q2 + mul_mod(q1, q1, p) + 2 * (p - q1)) % p;
    let inverse_two = p.div_ceil(2);
    let lerch_remainder = mul_mod(expression, inverse_two, p);
    let generalized_k = (q2 != 0).then(|| {
        let ratio = mul_mod(2 * lerch_remainder % p, inverse_mod(q2, p), p);
        if ratio <= 1 { 1 - ratio } else { p + 1 - ratio }
    });
    Invariants {
        p,
        primitive_root: g,
        q1,
        q2,
        q3: moments.q3.then_some(q3),
        q4: moments.q4.then_some(q4),
        lerch_remainder: Some(lerch_remainder),
        generalized_k,
        is_lerch: expression == 0,
        is_gy_exceptional: q2 == 0,
        is_wilson: q1 == 0,
        q1_equals_2: q1 == 2,
    }
}

/// Exposes the generated values for cross-checking the recurrence itself.
pub fn recurrence_values(p: u64, trial_primes: &[u64]) -> Vec<(u64, u64)> {
    assert!((3..=4_000_000_000).contains(&p));
    let g = primitive_root(p, trial_primes);
    let g_inverse = inverse_mod(g, p);
    let qg = fermat_quotient_mod_p(g, p);
    let mut out = Vec::with_capacity((p - 1) as usize);
    let (mut c, mut v, mut u) = (1u64, 1u64, 0u64);
    for j in 0..p - 1 {
        out.push((c, u));
        if j + 1 == p - 1 {
            break;
        }
        let product = g * c;
        let k = product / p;
        c = product - k * p;
        v = (v * g_inverse) % p;
        u = (u + qg + k * v) % p;
    }
    out
}
