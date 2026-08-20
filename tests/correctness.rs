use lerch_prime_search::arith::fermat_quotient_mod_p;
use lerch_prime_search::recurrence::{MomentOptions, recurrence_invariants, recurrence_values};
use lerch_prime_search::reference::direct_invariants;
use lerch_prime_search::sieve::{integer_sqrt, segmented_primes, simple_primes};
use lerch_prime_search::verify::{direct_lerch_remainder_bigint, verify_lerch_bigint};
use num_traits::Zero;

#[test]
fn every_recurrence_value_and_moment_below_1000_matches_definitions() {
    let base = simple_primes(integer_sqrt(1000));
    for p in segmented_primes(3, 999, &base) {
        let mut seen = vec![false; p as usize];
        for (a, q) in recurrence_values(p, &base) {
            assert!(!seen[a as usize], "duplicate a={a} at p={p}");
            seen[a as usize] = true;
            assert_eq!(q, fermat_quotient_mod_p(a, p), "p={p}, a={a}");
        }
        assert!(
            seen[1..].iter().all(|&yes| yes),
            "incomplete orbit at p={p}"
        );
        let fast = recurrence_invariants(p, &base, MomentOptions { q3: true, q4: true });
        let slow = direct_invariants(p, true, true);
        assert_eq!(fast.q1, slow.q1, "Q1 at p={p}");
        assert_eq!(fast.q2, slow.q2, "Q2 at p={p}");
        assert_eq!(fast.q3, slow.q3, "Q3 at p={p}");
        assert_eq!(fast.q4, slow.q4, "Q4 at p={p}");
        assert_eq!(fast.q1, slow.wilson, "Lerch congruence at p={p}");
        assert_eq!(
            fast.lerch_remainder,
            Some(slow.lerch_remainder),
            "L at p={p}"
        );
    }
}

#[test]
fn known_exceptional_primes_are_exact_below_5000() {
    let base = simple_primes(integer_sqrt(5000));
    let mut lerch = Vec::new();
    let mut gy = vec![2];
    let mut wilson = Vec::new();
    for p in segmented_primes(3, 4999, &base) {
        let value = recurrence_invariants(p, &base, MomentOptions::default());
        if value.is_lerch {
            lerch.push(p);
        }
        if value.is_gy_exceptional {
            gy.push(p);
        }
        if value.is_wilson {
            wilson.push(p);
        }
    }
    assert_eq!(lerch, [3, 103, 839, 2237]);
    assert_eq!(gy, [2, 11, 971]);
    assert_eq!(wilson, [5, 13, 563]);
}

#[test]
fn direct_lerch_quotient_identity_for_small_primes() {
    let base = simple_primes(20);
    for p in segmented_primes(3, 199, &base) {
        let fast = recurrence_invariants(p, &base, MomentOptions::default());
        assert_eq!(
            fast.lerch_remainder,
            Some(direct_lerch_remainder_bigint(p)),
            "p={p}"
        );
    }
}

#[test]
fn known_lerch_primes_pass_both_bigint_congruences() {
    for p in [3, 103, 839, 2237] {
        let transcript = verify_lerch_bigint(p);
        assert!(transcript.q1_minus_wilson_mod_p2.is_zero(), "p={p}");
        assert!(transcript.power_sum_residue.is_zero(), "p={p}");
    }
}

/// The release validation command runs this same definition-level comparison
/// over every prime below 100,000. Keep it opt-in because it is expensive.
#[test]
#[ignore = "run the release validate command with limit 100000"]
fn exhaustive_reference_validation_below_100000() {
    let base = simple_primes(integer_sqrt(100_000));
    for p in segmented_primes(3, 99_999, &base) {
        let fast = recurrence_invariants(p, &base, MomentOptions { q3: true, q4: true });
        let slow = direct_invariants(p, true, true);
        assert_eq!(
            (fast.q1, fast.q2, fast.q3, fast.q4),
            (slow.q1, slow.q2, slow.q3, slow.q4),
            "p={p}"
        );
        for (a, q) in recurrence_values(p, &base) {
            assert_eq!(q, fermat_quotient_mod_p(a, p), "p={p}, a={a}");
        }
    }
}
