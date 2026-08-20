use crate::arith::{fermat_quotient_mod_p, mul_mod};
use crate::recurrence::Invariants;
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationTranscript {
    pub p: u64,
    pub method: String,
    pub recurrence_q1: u64,
    pub recurrence_q2: u64,
    pub direct_q1_mod_p: u64,
    pub direct_q2_mod_p: u64,
    pub direct_q3_mod_p: Option<u64>,
    pub direct_q4_mod_p: Option<u64>,
    pub wilson_mod_p: u64,
    pub recurrence_matches_direct: bool,
    pub q1_matches_wilson_mod_p: bool,
    pub q1_minus_wilson_mod_p2: Option<String>,
    pub power_sum_minus_factorial_minus_p_mod_p3: Option<String>,
    pub lerch_verified: Option<bool>,
    pub verified: bool,
}

/// Definition-based verification for every rare condition. For Lerch hits this
/// additionally performs both independent bigint congruences modulo p^2/p^3.
pub fn verify_rare_candidate(fast: &Invariants) -> VerificationTranscript {
    let p = fast.p;
    if p == 2 {
        return VerificationTranscript {
            p,
            method: "definition special case".into(),
            recurrence_q1: 0,
            recurrence_q2: 0,
            direct_q1_mod_p: 0,
            direct_q2_mod_p: 0,
            direct_q3_mod_p: fast.q3.map(|_| 0),
            direct_q4_mod_p: fast.q4.map(|_| 0),
            wilson_mod_p: 1,
            recurrence_matches_direct: true,
            q1_matches_wilson_mod_p: false,
            q1_minus_wilson_mod_p2: None,
            power_sum_minus_factorial_minus_p_mod_p3: None,
            lerch_verified: None,
            verified: fast.is_gy_exceptional,
        };
    }
    let p2 = p * p;
    let mut q1 = 0u64;
    let mut q2 = 0u64;
    let mut q3 = 0u64;
    let mut q4 = 0u64;
    for a in 1..p {
        let q = fermat_quotient_mod_p(a, p);
        q1 = (q1 + q) % p;
        let square = mul_mod(q, q, p);
        q2 = (q2 + square) % p;
        if fast.q3.is_some() {
            q3 = (q3 + mul_mod(square, q, p)) % p;
        }
        if fast.q4.is_some() {
            q4 = (q4 + mul_mod(square, square, p)) % p;
        }
    }
    let mut factorial = 1u64;
    for a in 2..p {
        factorial = mul_mod(factorial, a, p2);
    }
    let wilson = ((factorial + 1) / p) % p;
    let direct_q3 = fast.q3.map(|_| q3);
    let direct_q4 = fast.q4.map(|_| q4);
    let recurrence_matches_direct =
        q1 == fast.q1 && q2 == fast.q2 && direct_q3 == fast.q3 && direct_q4 == fast.q4;
    let q1_matches_wilson_mod_p = q1 == wilson;

    let (q1_minus_wilson_mod_p2, power_residue, lerch_verified) = if fast.is_lerch {
        let full = verify_lerch_bigint(p);
        (
            Some(full.q1_minus_wilson_mod_p2.to_string()),
            Some(full.power_sum_residue.to_string()),
            Some(full.q1_minus_wilson_mod_p2.is_zero() && full.power_sum_residue.is_zero()),
        )
    } else {
        (None, None, None)
    };
    let verified =
        recurrence_matches_direct && q1_matches_wilson_mod_p && lerch_verified.unwrap_or(true);
    VerificationTranscript {
        p,
        method: if fast.is_lerch {
            "direct q_p definitions plus independent bigint p^3 power sum".into()
        } else {
            "direct q_p definitions and factorial modulo p^2".into()
        },
        recurrence_q1: fast.q1,
        recurrence_q2: fast.q2,
        direct_q1_mod_p: q1,
        direct_q2_mod_p: q2,
        direct_q3_mod_p: direct_q3,
        direct_q4_mod_p: direct_q4,
        wilson_mod_p: wilson,
        recurrence_matches_direct,
        q1_matches_wilson_mod_p,
        q1_minus_wilson_mod_p2,
        power_sum_minus_factorial_minus_p_mod_p3: power_residue,
        lerch_verified,
        verified,
    }
}

pub struct BigintLerchVerification {
    pub q1_minus_wilson_mod_p2: BigUint,
    pub power_sum_residue: BigUint,
}

pub fn verify_lerch_bigint(p: u64) -> BigintLerchVerification {
    assert!(p >= 3);
    let pb = BigUint::from(p);
    let p2 = &pb * &pb;
    let p3 = &p2 * &pb;
    let exponent = BigUint::from(p - 1);
    let mut q1_mod_p2 = BigUint::zero();
    let mut power_sum = BigUint::zero();
    for a in 1..p {
        let power = BigUint::from(a).modpow(&exponent, &p3);
        let q = (&power - BigUint::one()) / &pb;
        q1_mod_p2 = (q1_mod_p2 + &q) % &p2;
        power_sum = (power_sum + power) % &p3;
    }
    let mut factorial = BigUint::one();
    for a in 2..p {
        factorial = (factorial * a) % &p3;
    }
    let wilson_mod_p2 = ((&factorial + BigUint::one()) / &pb) % &p2;
    let q1_minus_wilson_mod_p2 = (&q1_mod_p2 + &p2 - wilson_mod_p2) % &p2;
    let power_sum_residue = (power_sum + &p3 - factorial + &p3 - &pb) % &p3;
    BigintLerchVerification {
        q1_minus_wilson_mod_p2,
        power_sum_residue,
    }
}

/// Direct Lerch quotient modulo p for validation, independently using p^3.
pub fn direct_lerch_remainder_bigint(p: u64) -> u64 {
    let pb = BigUint::from(p);
    let p2 = &pb * &pb;
    let p3 = &p2 * &pb;
    let exponent = BigUint::from(p - 1);
    let mut q1_mod_p2 = BigUint::zero();
    for a in 1..p {
        let power = BigUint::from(a).modpow(&exponent, &p3);
        q1_mod_p2 = (q1_mod_p2 + (power - BigUint::one()) / &pb) % &p2;
    }
    let mut factorial = BigUint::one();
    for a in 2..p {
        factorial = (factorial * a) % &p3;
    }
    let wilson_mod_p2 = ((factorial + BigUint::one()) / &pb) % &p2;
    let difference = (q1_mod_p2 + &p2 - wilson_mod_p2) % &p2;
    (&difference / &pb).to_u64().expect("residue fits u64")
}
