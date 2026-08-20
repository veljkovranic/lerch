use crate::recurrence::{Invariants, MomentOptions, recurrence_invariants};
use crate::sieve::{integer_sqrt, segmented_primes, simple_primes};
use crate::verify::{VerificationTranscript, verify_rare_candidate};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub const FORMAT_VERSION: &str = "lerch-prime-search-v1";

#[derive(Clone, Debug)]
pub struct SearchOptions {
    pub start: u64,
    pub end: u64,
    pub chunk_size: u64,
    pub threads: usize,
    pub output_dir: PathBuf,
    pub resume: bool,
    pub moments: MomentOptions,
    pub sample_every: u64,
    pub verify_rare: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrimeRecord {
    pub sample: bool,
    pub invariants: Invariants,
    pub verification_file: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SegmentResult {
    pub version: String,
    pub status: String,
    pub start: u64,
    pub end: u64,
    pub configuration_sha256: String,
    pub result_sha256: String,
    pub prime_count: u64,
    pub sum_of_p_processed: String,
    pub recurrence_steps: String,
    pub elapsed_seconds: f64,
    pub lerch_hits: Vec<u64>,
    pub q2_zero_hits: Vec<u64>,
    pub wilson_hits: Vec<u64>,
    pub q1_equals_2_hits: Vec<u64>,
    pub q3_zero_hits: Vec<u64>,
    pub q4_zero_hits: Vec<u64>,
    pub records: Vec<PrimeRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub status: String,
    pub requested_start: u64,
    pub requested_end: u64,
    pub completed_intervals: Vec<[u64; 2]>,
    pub prime_count: u64,
    pub sum_of_p_processed: String,
    pub lerch_hits: Vec<u64>,
    pub q2_zero_hits: Vec<u64>,
    pub wilson_hits: Vec<u64>,
    pub q1_equals_2_hits: Vec<u64>,
    pub q3_zero_hits: Vec<u64>,
    pub q4_zero_hits: Vec<u64>,
    pub segment_files: Vec<String>,
    pub manifest_sha256: String,
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn configuration_hash(options: &SearchOptions, start: u64, end: u64) -> String {
    hex_sha256(
        format!(
            "{FORMAT_VERSION}:{start}:{end}:q3={}:q4={}:sample={}:verify={}",
            options.moments.q3, options.moments.q4, options.sample_every, options.verify_rare
        )
        .as_bytes(),
    )
}

fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = fs::File::create(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(temp, path)
}

fn segment_path(dir: &Path, index: usize) -> PathBuf {
    dir.join("segments")
        .join(format!("segment_{index:08}.json"))
}

fn verification_path(dir: &Path, p: u64) -> PathBuf {
    dir.join("verifications").join(format!("p_{p}.json"))
}

fn read_completed(path: &Path, expected_hash: &str) -> Option<SegmentResult> {
    let text = fs::read_to_string(path).ok()?;
    let result: SegmentResult = serde_json::from_str(&text).ok()?;
    let expected_result_hash = segment_digest(&result).ok()?;
    (result.status == "complete"
        && result.configuration_sha256 == expected_hash
        && result.result_sha256 == expected_result_hash)
        .then_some(result)
}

fn segment_digest(result: &SegmentResult) -> Result<String, serde_json::Error> {
    serde_json::to_vec(&(
        result.start,
        result.end,
        result.prime_count as usize,
        &result.sum_of_p_processed,
        &result.lerch_hits,
        &result.q2_zero_hits,
        &result.wilson_hits,
        &result.q1_equals_2_hits,
        &result.q3_zero_hits,
        &result.q4_zero_hits,
        &result.records,
    ))
    .map(|bytes| hex_sha256(&bytes))
}

fn save_verification(dir: &Path, transcript: &VerificationTranscript) -> Result<String, String> {
    if !transcript.verified {
        return Err(format!(
            "independent verification failed for p={}",
            transcript.p
        ));
    }
    let path = verification_path(dir, transcript.p);
    let bytes = serde_json::to_vec_pretty(transcript).map_err(|e| e.to_string())?;
    atomic_write(&path, &bytes).map_err(|e| e.to_string())?;
    Ok(path
        .strip_prefix(dir)
        .unwrap_or(&path)
        .to_string_lossy()
        .into_owned())
}

fn scan_segment(
    index: usize,
    start: u64,
    end: u64,
    options: &SearchOptions,
    base_primes: &[u64],
) -> Result<SegmentResult, String> {
    let config_hash = configuration_hash(options, start, end);
    let path = segment_path(&options.output_dir, index);
    if options.resume
        && let Some(stored) = read_completed(&path, &config_hash)
    {
        return Ok(stored);
    }
    let timer = Instant::now();
    let primes = segmented_primes(start, end, base_primes);
    let mut records = Vec::new();
    let mut lerch_hits = Vec::new();
    let mut q2_zero_hits = Vec::new();
    let mut wilson_hits = Vec::new();
    let mut q1_equals_2_hits = Vec::new();
    let mut q3_zero_hits = Vec::new();
    let mut q4_zero_hits = Vec::new();
    let mut sum_p = 0u128;
    let mut steps = 0u128;
    for (ordinal, p) in primes.iter().copied().enumerate() {
        sum_p += p as u128;
        steps += p.saturating_sub(1) as u128;
        let invariants = recurrence_invariants(p, base_primes, options.moments);
        if invariants.is_lerch {
            lerch_hits.push(p);
        }
        if invariants.is_gy_exceptional {
            q2_zero_hits.push(p);
        }
        if invariants.is_wilson {
            wilson_hits.push(p);
        }
        if invariants.q1_equals_2 {
            q1_equals_2_hits.push(p);
        }
        if invariants.q3 == Some(0) {
            q3_zero_hits.push(p);
        }
        if invariants.q4 == Some(0) {
            q4_zero_hits.push(p);
        }
        let rare = invariants.rare();
        let verification_file = if rare && options.verify_rare {
            Some(save_verification(
                &options.output_dir,
                &verify_rare_candidate(&invariants),
            )?)
        } else {
            None
        };
        let sample = options.sample_every != 0 && ordinal as u64 % options.sample_every == 0;
        if rare || sample {
            records.push(PrimeRecord {
                sample,
                invariants,
                verification_file,
            });
        }
    }
    let mut result = SegmentResult {
        version: FORMAT_VERSION.into(),
        status: "complete".into(),
        start,
        end,
        configuration_sha256: config_hash,
        result_sha256: String::new(),
        prime_count: primes.len() as u64,
        sum_of_p_processed: sum_p.to_string(),
        recurrence_steps: steps.to_string(),
        elapsed_seconds: timer.elapsed().as_secs_f64(),
        lerch_hits,
        q2_zero_hits,
        wilson_hits,
        q1_equals_2_hits,
        q3_zero_hits,
        q4_zero_hits,
        records,
    };
    result.result_sha256 = segment_digest(&result).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?;
    atomic_write(&path, &bytes).map_err(|e| e.to_string())?;
    eprintln!(
        "completed [{start}, {end}]: {} primes in {:.3}s",
        result.prime_count, result.elapsed_seconds
    );
    Ok(result)
}

pub fn run_search(options: SearchOptions) -> Result<Manifest, String> {
    if options.start > options.end {
        return Err("--start must not exceed --end".into());
    }
    if options.end > 4_000_000_000 {
        return Err("--end exceeds the fixed-width limit 4000000000".into());
    }
    if options.chunk_size == 0 || options.threads == 0 {
        return Err("--chunk-size and --threads must be positive".into());
    }
    fs::create_dir_all(&options.output_dir).map_err(|e| e.to_string())?;
    let base_primes = simple_primes(integer_sqrt(options.end));
    let mut chunks = Vec::new();
    let mut start = options.start;
    while start <= options.end {
        let end = options
            .end
            .min(start.saturating_add(options.chunk_size - 1));
        chunks.push((chunks.len(), start, end));
        if end == u64::MAX {
            break;
        }
        start = end + 1;
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(options.threads)
        .build()
        .map_err(|e| e.to_string())?;
    let mut segments: Vec<SegmentResult> = pool.install(|| {
        chunks
            .par_iter()
            .map(|&(index, start, end)| scan_segment(index, start, end, &options, &base_primes))
            .collect::<Result<Vec<_>, _>>()
    })?;
    segments.sort_by_key(|s| s.start);
    let mut prime_count = 0u64;
    let mut sum_p = 0u128;
    let mut lerch_hits = Vec::new();
    let mut q2_zero_hits = Vec::new();
    let mut wilson_hits = Vec::new();
    let mut q1_equals_2_hits = Vec::new();
    let mut q3_zero_hits = Vec::new();
    let mut q4_zero_hits = Vec::new();
    let mut segment_files = Vec::new();
    for (index, segment) in segments.iter().enumerate() {
        prime_count += segment.prime_count;
        sum_p += segment
            .sum_of_p_processed
            .parse::<u128>()
            .map_err(|e| e.to_string())?;
        lerch_hits.extend_from_slice(&segment.lerch_hits);
        q2_zero_hits.extend_from_slice(&segment.q2_zero_hits);
        wilson_hits.extend_from_slice(&segment.wilson_hits);
        q1_equals_2_hits.extend_from_slice(&segment.q1_equals_2_hits);
        q3_zero_hits.extend_from_slice(&segment.q3_zero_hits);
        q4_zero_hits.extend_from_slice(&segment.q4_zero_hits);
        segment_files.push(
            segment_path(&options.output_dir, index)
                .strip_prefix(&options.output_dir)
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        );
    }
    let hash_input = segments
        .iter()
        .map(|s| s.result_sha256.as_str())
        .collect::<Vec<_>>()
        .join(":");
    let manifest = Manifest {
        version: FORMAT_VERSION.into(),
        status: "complete".into(),
        requested_start: options.start,
        requested_end: options.end,
        completed_intervals: segments.iter().map(|s| [s.start, s.end]).collect(),
        prime_count,
        sum_of_p_processed: sum_p.to_string(),
        lerch_hits,
        q2_zero_hits,
        wilson_hits,
        q1_equals_2_hits,
        q3_zero_hits,
        q4_zero_hits,
        segment_files,
        manifest_sha256: hex_sha256(hash_input.as_bytes()),
    };
    let bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    atomic_write(&options.output_dir.join("manifest.json"), &bytes).map_err(|e| e.to_string())?;
    Ok(manifest)
}
