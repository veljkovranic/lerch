use lerch_prime_search::recurrence::{MomentOptions, recurrence_invariants, recurrence_values};
use lerch_prime_search::reduction::{Barrett32, Montgomery32};
use lerch_prime_search::reference::{direct_invariants, direct_values};
use lerch_prime_search::search::{SearchOptions, run_search};
use lerch_prime_search::sieve::{integer_sqrt, segmented_primes, simple_primes};
use lerch_prime_search::verify::{direct_lerch_remainder_bigint, verify_rare_candidate};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::path::PathBuf;
use std::time::Instant;

fn usage() -> &'static str {
    "lerch-prime-search <command> [options]\n\
     commands:\n\
       search --start N --end N [--chunk-size N] [--threads N] [--output-dir DIR]\n\
              [--resume] [--q3] [--q4] [--sample-every N] [--no-verify-rare]\n\
       validate [--limit N] [--bigint-limit N]\n\
       verify --prime P\n\
       benchmark [--primes P1,P2,...] [--direct-max N]\n\
       reducer-benchmark [--prime P] [--iterations N]"
}

fn value(args: &[String], name: &str) -> Result<Option<String>, String> {
    let Some(index) = args.iter().position(|a| a == name) else {
        return Ok(None);
    };
    args.get(index + 1)
        .cloned()
        .map(Some)
        .ok_or_else(|| format!("missing value for {name}"))
}

fn parsed<T: std::str::FromStr>(args: &[String], name: &str, default: T) -> Result<T, String> {
    value(args, name)?
        .map(|s| s.parse().map_err(|_| format!("invalid {name}: {s}")))
        .unwrap_or(Ok(default))
}

fn reject_unknown(args: &[String], valued: &[&str], flags: &[&str]) -> Result<(), String> {
    let mut i = 0;
    while i < args.len() {
        if valued.contains(&args[i].as_str()) {
            i += 2;
        } else if flags.contains(&args[i].as_str()) {
            i += 1;
        } else {
            return Err(format!("unknown option: {}", args[i]));
        }
    }
    Ok(())
}

fn search(args: &[String]) -> Result<(), String> {
    reject_unknown(
        args,
        &[
            "--start",
            "--end",
            "--chunk-size",
            "--threads",
            "--output-dir",
            "--sample-every",
        ],
        &["--resume", "--q3", "--q4", "--no-verify-rare"],
    )?;
    let start = parsed(args, "--start", 2u64)?;
    let end = value(args, "--end")?
        .ok_or("search requires --end")?
        .parse()
        .map_err(|_| "invalid --end")?;
    let options = SearchOptions {
        start,
        end,
        chunk_size: parsed(args, "--chunk-size", 100_000u64)?,
        threads: parsed(
            args,
            "--threads",
            std::thread::available_parallelism().map_or(1, |n| n.get()),
        )?,
        output_dir: PathBuf::from(
            value(args, "--output-dir")?.unwrap_or_else(|| format!("results/{start}_{end}")),
        ),
        resume: args.iter().any(|a| a == "--resume"),
        moments: MomentOptions {
            q3: args.iter().any(|a| a == "--q3"),
            q4: args.iter().any(|a| a == "--q4"),
        },
        sample_every: parsed(args, "--sample-every", 0u64)?,
        verify_rare: !args.iter().any(|a| a == "--no-verify-rare"),
    };
    let manifest = run_search(options)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn validate(args: &[String]) -> Result<(), String> {
    reject_unknown(args, &["--limit", "--bigint-limit"], &[])?;
    let limit = parsed(args, "--limit", 100_000u64)?;
    let bigint_limit = parsed(args, "--bigint-limit", 300u64)?.min(limit);
    let base = simple_primes(integer_sqrt(limit));
    let primes = segmented_primes(3, limit.saturating_sub(1), &base);
    let timer = Instant::now();
    let mut lerch = Vec::new();
    let mut gy = vec![2u64];
    let mut wilson = Vec::new();
    for (index, p) in primes.iter().copied().enumerate() {
        let fast = recurrence_invariants(p, &base, MomentOptions { q3: true, q4: true });
        let direct = direct_invariants(p, true, true);
        if fast.q1 != direct.q1
            || fast.q2 != direct.q2
            || fast.q3 != direct.q3
            || fast.q4 != direct.q4
            || fast.q1 != direct.wilson
            || fast.lerch_remainder != Some(direct.lerch_remainder)
        {
            return Err(format!(
                "aggregate mismatch at p={p}: fast={fast:?}, direct={direct:?}"
            ));
        }
        let generated = recurrence_values(p, &base);
        let values = direct_values(p);
        for (a, q) in generated {
            if q != values[(a - 1) as usize] {
                return Err(format!("q_p(a) mismatch at p={p}, a={a}"));
            }
        }
        if p <= bigint_limit {
            let direct_l = direct_lerch_remainder_bigint(p);
            if fast.lerch_remainder != Some(direct_l) {
                return Err(format!("direct Lerch quotient mismatch at p={p}"));
            }
        }
        if fast.is_lerch {
            lerch.push(p);
        }
        if fast.is_gy_exceptional {
            gy.push(p);
        }
        if fast.is_wilson {
            wilson.push(p);
        }
        if index % 100 == 0 {
            eprint!("\rvalidated {}/{} primes", index + 1, primes.len());
        }
    }
    eprintln!();
    let expected_lerch: Vec<u64> = [3, 103, 839, 2237]
        .into_iter()
        .filter(|&p| p < limit)
        .collect();
    if lerch != expected_lerch {
        return Err(format!("unexpected Lerch list: {lerch:?}"));
    }
    println!(
        "validated {} odd primes below {} in {:.3}s; Lerch={:?}; Gy={:?}; Wilson={:?}",
        primes.len(),
        limit,
        timer.elapsed().as_secs_f64(),
        lerch,
        gy,
        wilson
    );
    Ok(())
}

fn verify(args: &[String]) -> Result<(), String> {
    reject_unknown(args, &["--prime"], &[])?;
    let p: u64 = value(args, "--prime")?
        .ok_or("verify requires --prime")?
        .parse()
        .map_err(|_| "invalid --prime")?;
    let base = simple_primes(integer_sqrt(p));
    if segmented_primes(p, p, &base) != [p] {
        return Err(format!("{p} is not prime"));
    }
    let fast = recurrence_invariants(p, &base, MomentOptions { q3: true, q4: true });
    let transcript = verify_rare_candidate(&fast);
    println!(
        "{}",
        serde_json::to_string_pretty(&transcript).map_err(|e| e.to_string())?
    );
    if transcript.verified {
        Ok(())
    } else {
        Err("verification failed".into())
    }
}

fn direct_power_sum_seconds(p: u64) -> f64 {
    let pb = BigUint::from(p);
    let modulus = &pb * &pb * &pb;
    let exponent = BigUint::from(p - 1);
    let timer = Instant::now();
    let mut sum = BigUint::zero();
    for a in 1..p {
        sum = (sum + BigUint::from(a).modpow(&exponent, &modulus)) % &modulus;
    }
    std::hint::black_box(sum + BigUint::one());
    timer.elapsed().as_secs_f64()
}

fn benchmark(args: &[String]) -> Result<(), String> {
    reject_unknown(args, &["--primes", "--direct-max"], &[])?;
    let list = value(args, "--primes")?.unwrap_or_else(|| "10007,100003,1000003".into());
    let primes: Vec<u64> = list
        .split(',')
        .map(|s| s.parse().map_err(|_| format!("invalid prime {s}")))
        .collect::<Result<_, _>>()?;
    let direct_max = parsed(args, "--direct-max", 20_000u64)?;
    println!(
        "p,primitive_root,steps,recurrence_seconds,ns_per_step,effective_multiplies_per_second,direct_q_seconds,direct_power_sum_seconds,speedup_vs_direct_q,speedup_vs_power_sum"
    );
    for p in primes {
        let base = simple_primes(integer_sqrt(p));
        if segmented_primes(p, p, &base) != [p] {
            return Err(format!("{p} is not prime"));
        }
        let timer = Instant::now();
        let fast = recurrence_invariants(p, &base, MomentOptions::default());
        let recurrence_seconds = timer.elapsed().as_secs_f64();
        let (direct_q, direct_power) = if p <= direct_max {
            let timer = Instant::now();
            std::hint::black_box(direct_invariants(p, false, false));
            let q = timer.elapsed().as_secs_f64();
            (Some(q), Some(direct_power_sum_seconds(p)))
        } else {
            (None, None)
        };
        let fmt = |v: Option<f64>| v.map(|x| format!("{x:.9}")).unwrap_or_default();
        println!(
            "{},{},{},{:.9},{:.3},{:.0},{},{},{},{}",
            p,
            fast.primitive_root,
            p - 1,
            recurrence_seconds,
            recurrence_seconds * 1e9 / (p - 1) as f64,
            4.0 * (p - 1) as f64 / recurrence_seconds,
            fmt(direct_q),
            fmt(direct_power),
            fmt(direct_q.map(|x| x / recurrence_seconds)),
            fmt(direct_power.map(|x| x / recurrence_seconds))
        );
    }
    Ok(())
}

fn reducer_benchmark(args: &[String]) -> Result<(), String> {
    reject_unknown(args, &["--prime", "--iterations"], &[])?;
    let p = parsed(args, "--prime", 1_000_003u64)?;
    let iterations = parsed(args, "--iterations", 10_000_000u64)?;
    if p >= 1u64 << 32 || p & 1 == 0 {
        return Err("--prime must be an odd prime below 2^32".into());
    }
    let base = simple_primes(integer_sqrt(p));
    if segmented_primes(p, p, &base) != [p] {
        return Err(format!("{p} is not prime"));
    }
    let barrett = Barrett32::new(p);
    let montgomery = Montgomery32::new(p);
    let mut a = 123_457 % p;
    let b = 765_431 % p;

    let timer = Instant::now();
    for _ in 0..iterations {
        a = std::hint::black_box(((a as u128 * b as u128) % p as u128) as u64);
    }
    let native = timer.elapsed().as_secs_f64();

    a = 123_457 % p;
    let timer = Instant::now();
    for _ in 0..iterations {
        a = std::hint::black_box(barrett.multiply(a, b));
    }
    let barrett_seconds = timer.elapsed().as_secs_f64();

    let mut am = montgomery.encode(123_457 % p);
    let bm = montgomery.encode(b);
    let timer = Instant::now();
    for _ in 0..iterations {
        am = std::hint::black_box(montgomery.multiply(am, bm));
    }
    std::hint::black_box(montgomery.decode(am));
    let montgomery_seconds = timer.elapsed().as_secs_f64();
    println!("method,seconds,ns_per_multiply");
    println!(
        "u128_remainder,{native:.9},{:.3}",
        native * 1e9 / iterations as f64
    );
    println!(
        "barrett,{barrett_seconds:.9},{:.3}",
        barrett_seconds * 1e9 / iterations as f64
    );
    println!(
        "montgomery_encoded,{montgomery_seconds:.9},{:.3}",
        montgomery_seconds * 1e9 / iterations as f64
    );
    Ok(())
}

fn main() {
    let mut argv = std::env::args().skip(1);
    let Some(command) = argv.next() else {
        eprintln!("{}", usage());
        std::process::exit(2);
    };
    let args: Vec<String> = argv.collect();
    let result = match command.as_str() {
        "search" => search(&args),
        "validate" => validate(&args),
        "verify" => verify(&args),
        "benchmark" => benchmark(&args),
        "reducer-benchmark" => reducer_benchmark(&args),
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            Ok(())
        }
        _ => Err(format!("unknown command: {command}\n{}", usage())),
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
