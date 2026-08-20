#!/usr/bin/env python3
"""Resumable exact search for prime-valued Lerch quotients.

For each odd prime p in an inclusive interval, PARI/GP constructs

    ell_p = (sum(a^(p-1), a=1..p-1) - p - (p-1)!)/p^2

exactly.  A primorial GCD finds an auditable small factor cheaply.  Survivors
receive a base-2 Fermat compositeness test and then PARI's BPSW-style
ispseudoprime test.  Only probable-prime survivors are retained in full.

Every per-prime JSON result is written atomically and validated before resume.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
from typing import Any


FORMAT_VERSION = "lerch-prime-quotient-search-v1"
FORMULA = "(sum(a^(p-1),a=1..p-1)-p-(p-1)!)/p^2"
FINGERPRINT_MODULI = [1_000_000_007, 1_000_000_009, 2_147_483_647]


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def atomic_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(fd, "w") as handle:
            json.dump(value, handle, indent=2, sort_keys=True)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    except BaseException:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass
        raise


def primes_in_range(start: int, end: int) -> list[int]:
    if end < 2 or start > end:
        return []
    sieve = bytearray(b"\x01") * (end + 1)
    sieve[0:2] = b"\x00\x00"
    for p in range(2, math.isqrt(end) + 1):
        if sieve[p]:
            sieve[p * p :: p] = b"\x00" * (((end - p * p) // p) + 1)
    return [p for p in range(max(3, start | 1), end + 1, 2) if sieve[p]]


def result_digest(result: dict[str, Any]) -> str:
    unhashed = {key: value for key, value in result.items() if key != "result_sha256"}
    return sha256_bytes(canonical_bytes(unhashed))


def completed_result(path: Path, p: int, configuration_sha256: str) -> dict[str, Any] | None:
    try:
        value = json.loads(path.read_text())
    except (FileNotFoundError, json.JSONDecodeError, OSError):
        return None
    if (
        value.get("version") != FORMAT_VERSION
        or value.get("p") != p
        or value.get("configuration_sha256") != configuration_sha256
        or value.get("result_sha256") != result_digest(value)
    ):
        return None
    if value.get("status") == "probable_prime":
        candidate = value.get("candidate")
        if not isinstance(candidate, dict) or not isinstance(candidate.get("path"), str):
            return None
        candidate_path = path.parent.parent / candidate["path"]
        try:
            if (
                not candidate_path.is_file()
                or candidate_path.stat().st_size != candidate.get("bytes")
                or sha256_file(candidate_path) != candidate.get("sha256")
            ):
                return None
        except OSError:
            return None
    return value


def gp_string(value: str) -> str:
    return json.dumps(value)


def boundary_certificate(args: argparse.Namespace) -> dict[str, Any] | None:
    if not args.target_digits:
        return None
    program = (
        f"p={args.end_prime};target={args.target_digits};"
        "lower=(p-1)^(p-1)-(p-1)!-p;"
        "print(lower>=10^target*p^2);\n"
    )
    completed = subprocess.run(
        [args.gp, "-fq", "-s", str(args.gp_stack_bytes)],
        input=program,
        text=True,
        capture_output=True,
        check=True,
    )
    passed = completed.stdout.strip().splitlines()[-1] == "1"
    return {
        "endpoint": args.end_prime,
        "target_decimal_digits": args.target_digits,
        "lower_bound": "((p-1)^(p-1)-(p-1)!-p)/p^2",
        "lower_bound_exceeds_or_equals_10_to_target": passed,
    }


class ActiveProcesses:
    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._processes: set[subprocess.Popen[str]] = set()

    def add(self, process: subprocess.Popen[str]) -> None:
        with self._lock:
            self._processes.add(process)

    def remove(self, process: subprocess.Popen[str]) -> None:
        with self._lock:
            self._processes.discard(process)

    def terminate_all(self) -> None:
        with self._lock:
            processes = list(self._processes)
        for process in processes:
            if process.poll() is None:
                process.terminate()


def parse_result_line(stdout: str) -> dict[str, Any]:
    lines = [line for line in stdout.splitlines() if line.startswith("LQRESULT|")]
    if len(lines) != 1:
        raise RuntimeError(f"expected one LQRESULT line, received {len(lines)}")
    fields = lines[0].split("|", 12)
    if len(fields) != 12:
        raise RuntimeError(f"malformed LQRESULT line: {lines[0][:200]}")
    (
        _, p, digits, build_ms, screen_ms, status, evidence, mod_p,
        mod_1, mod_2, mod_3, inline_value,
    ) = fields
    return {
        "p": int(p),
        "decimal_digits": int(digits),
        "build_milliseconds": int(build_ms),
        "screen_milliseconds": int(screen_ms),
        "status": status,
        "evidence": int(evidence),
        "lerch_quotient_mod_p": int(mod_p),
        "fingerprints": {
            str(FINGERPRINT_MODULI[0]): int(mod_1),
            str(FINGERPRINT_MODULI[1]): int(mod_2),
            str(FINGERPRINT_MODULI[2]): int(mod_3),
        },
        "inline_lerch_quotient": int(inline_value) if inline_value else None,
    }


def run_one(
    p: int,
    args: argparse.Namespace,
    worker_path: Path,
    worker_sha256: str,
    gp_version: str,
    configuration_sha256: str,
    results_dir: Path,
    candidates_dir: Path,
    active: ActiveProcesses,
) -> tuple[dict[str, Any], bool]:
    result_path = results_dir / f"p_{p}.json"
    if args.resume:
        stored = completed_result(result_path, p, configuration_sha256)
        if stored is not None:
            return stored, True

    candidates_dir.mkdir(parents=True, exist_ok=True)
    fd, temporary_candidate_name = tempfile.mkstemp(
        prefix=f".p_{p}.", suffix=".txt", dir=candidates_dir
    )
    os.close(fd)
    os.unlink(temporary_candidate_name)
    temporary_candidate = Path(temporary_candidate_name)
    final_candidate = candidates_dir / f"p_{p}.txt"

    gp_program = (
        f"default(parisizemax,{args.gp_max_stack_bytes});\n"
        f"read({gp_string(str(worker_path))});\n"
        f"lq_search_one({p},{args.trial_bound},{args.inline_digits},"
        f"{gp_string(str(temporary_candidate))});\n"
    )
    started = time.monotonic()
    process = subprocess.Popen(
        [args.gp, "-fq", "-s", str(args.gp_stack_bytes)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    active.add(process)
    try:
        try:
            stdout, stderr = process.communicate(
                gp_program,
                timeout=args.timeout_seconds if args.timeout_seconds else None,
            )
        except subprocess.TimeoutExpired:
            process.kill()
            stdout, stderr = process.communicate()
            raise RuntimeError(f"PARI/GP timed out for p={p}")
    finally:
        active.remove(process)
    wall_seconds = time.monotonic() - started
    if process.returncode != 0:
        temporary_candidate.unlink(missing_ok=True)
        detail = stderr.strip().splitlines()[-1] if stderr.strip() else "no stderr"
        raise RuntimeError(f"PARI/GP failed for p={p}: {detail}")

    parsed = parse_result_line(stdout)
    if parsed["p"] != p:
        raise RuntimeError(f"PARI/GP returned p={parsed['p']} while processing p={p}")

    candidate_metadata = None
    if parsed["status"] == "probable_prime":
        if not temporary_candidate.exists():
            raise RuntimeError(f"probable-prime quotient for p={p} was not retained")
        os.replace(temporary_candidate, final_candidate)
        candidate_metadata = {
            "path": str(final_candidate.relative_to(args.output_dir)),
            "sha256": sha256_file(final_candidate),
            "bytes": final_candidate.stat().st_size,
        }
    else:
        temporary_candidate.unlink(missing_ok=True)
        final_candidate.unlink(missing_ok=True)

    result: dict[str, Any] = {
        "version": FORMAT_VERSION,
        "formula": FORMULA,
        "p": p,
        "decimal_digits": parsed["decimal_digits"],
        "status": parsed["status"],
        "evidence": parsed["evidence"],
        "evidence_description": {
            "composite_factor": "a proper prime divisor from gcd(ell_p, primorial)",
            "composite_fermat": "base a has a^(ell_p-1) != 1 (mod ell_p)",
            "composite_bpsw": "PARI ispseudoprime returned false after base-2 passed",
            "prime_proven": "small quotient proved prime by PARI isprime",
            "probable_prime": "PARI ispseudoprime returned true; independent proof required",
            "nonprime_zero": "the quotient is zero",
            "nonprime_one": "the quotient is one",
        }[parsed["status"]],
        "lerch_quotient_mod_p": parsed["lerch_quotient_mod_p"],
        "fingerprints": parsed["fingerprints"],
        "inline_lerch_quotient": parsed["inline_lerch_quotient"],
        "trial_division_bound": args.trial_bound,
        "build_milliseconds": parsed["build_milliseconds"],
        "screen_milliseconds": parsed["screen_milliseconds"],
        "wall_seconds": wall_seconds,
        "gp_version": gp_version,
        "worker_sha256": worker_sha256,
        "configuration_sha256": configuration_sha256,
        "candidate": candidate_metadata,
    }
    result["result_sha256"] = result_digest(result)
    atomic_json(result_path, result)
    return result, False


def main() -> int:
    script_dir = Path(__file__).resolve().parent
    crate_dir = script_dir.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--start-prime", type=int, default=3)
    parser.add_argument("--end-prime", type=int, default=100_003)
    parser.add_argument("--workers", type=int, default=max(1, (os.cpu_count() or 2) // 2))
    parser.add_argument("--trial-bound", type=int, default=10_000)
    parser.add_argument("--inline-digits", type=int, default=1_000)
    parser.add_argument("--gp", default="gp")
    parser.add_argument("--gp-stack-bytes", type=int, default=64_000_000)
    parser.add_argument("--gp-max-stack-bytes", type=int, default=4_000_000_000)
    parser.add_argument("--timeout-seconds", type=int, default=0)
    parser.add_argument(
        "--target-digits",
        type=int,
        default=0,
        help="certify that the endpoint quotient is above this decimal-digit boundary",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=crate_dir / "results" / "prime-lerch-quotients-3-100003",
    )
    parser.add_argument("--resume", action=argparse.BooleanOptionalAction, default=True)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--limit", type=int, default=0, help="process only the first N primes")
    args = parser.parse_args()

    if args.start_prime > args.end_prime:
        parser.error("--start-prime must not exceed --end-prime")
    if args.workers < 1 or args.trial_bound < 2:
        parser.error("--workers must be positive and --trial-bound must be at least 2")
    if shutil.which(args.gp) is None:
        parser.error(f"PARI/GP executable not found: {args.gp}")

    args.output_dir = args.output_dir.resolve()
    driver_sha256 = sha256_file(Path(__file__).resolve())
    worker_path = script_dir / "lerch_quotient_worker.gp"
    worker_sha256 = sha256_file(worker_path)
    gp_version = subprocess.check_output(
        [args.gp, "--version-short"], text=True
    ).strip()
    configuration = {
        "version": FORMAT_VERSION,
        "formula": FORMULA,
        "start_prime": args.start_prime,
        "end_prime": args.end_prime,
        "trial_bound": args.trial_bound,
        "inline_digits": args.inline_digits,
        "gp_version": gp_version,
        "driver_sha256": driver_sha256,
        "worker_sha256": worker_sha256,
        "fingerprint_moduli": FINGERPRINT_MODULI,
        "limit": args.limit,
        "target_digits": args.target_digits,
    }
    configuration_sha256 = sha256_bytes(canonical_bytes(configuration))
    primes = primes_in_range(args.start_prime, args.end_prime)
    if args.limit:
        primes = primes[: args.limit]
    print(
        f"prime inputs: {len(primes)}; inclusive range "
        f"[{args.start_prime}, {args.end_prime}]; workers={args.workers}",
        flush=True,
    )
    print(
        f"PARI/GP {gp_version}; trial primorial <= {args.trial_bound}; "
        f"configuration {configuration_sha256}",
        flush=True,
    )
    if args.dry_run:
        certificate = boundary_certificate(args)
        if certificate is not None:
            print(json.dumps(certificate, indent=2, sort_keys=True))
        return 0

    results_dir = args.output_dir / "primes"
    candidates_dir = args.output_dir / "candidates"
    results_dir.mkdir(parents=True, exist_ok=True)
    active = ActiveProcesses()
    stop = threading.Event()
    previous_handlers: dict[int, Any] = {}

    def handle_signal(signum: int, _frame: Any) -> None:
        stop.set()
        active.terminate_all()
        previous = previous_handlers.get(signum)
        if callable(previous):
            signal.signal(signum, previous)

    for signum in (signal.SIGINT, signal.SIGTERM):
        previous_handlers[signum] = signal.getsignal(signum)
        signal.signal(signum, handle_signal)

    completed: list[dict[str, Any]] = []
    failures: list[tuple[int, str]] = []
    started = time.monotonic()
    executor = concurrent.futures.ThreadPoolExecutor(max_workers=args.workers)
    futures: dict[concurrent.futures.Future[tuple[dict[str, Any], bool]], int] = {}
    try:
        for p in primes:
            if stop.is_set():
                break
            future = executor.submit(
                run_one,
                p,
                args,
                worker_path,
                worker_sha256,
                gp_version,
                configuration_sha256,
                results_dir,
                candidates_dir,
                active,
            )
            futures[future] = p
        for future in concurrent.futures.as_completed(futures):
            p = futures[future]
            if stop.is_set():
                break
            try:
                result, resumed = future.result()
            except Exception as exc:
                failures.append((p, str(exc)))
                print(f"FAILED p={p}: {exc}", file=sys.stderr, flush=True)
                stop.set()
                active.terminate_all()
                break
            completed.append(result)
            prefix = "resumed" if resumed else "completed"
            print(
                f"{prefix} p={p}: digits={result['decimal_digits']} "
                f"status={result['status']} wall={result['wall_seconds']:.3f}s",
                flush=True,
            )
    finally:
        if stop.is_set():
            for future in futures:
                future.cancel()
            active.terminate_all()
        executor.shutdown(wait=True, cancel_futures=True)
        for signum, previous in previous_handlers.items():
            signal.signal(signum, previous)

    if failures:
        return 1
    if stop.is_set() or len(completed) != len(primes):
        print("search interrupted; completed prime files are safe to resume", file=sys.stderr)
        return 130

    completed.sort(key=lambda item: item["p"])
    counts: dict[str, int] = {}
    for result in completed:
        counts[result["status"]] = counts.get(result["status"], 0) + 1
    aggregate = ":".join(result["result_sha256"] for result in completed)
    certificate = boundary_certificate(args) if not args.limit else None
    if certificate is not None and not certificate["lower_bound_exceeds_or_equals_10_to_target"]:
        print(
            "endpoint lower bound does not cross --target-digits; choose a larger endpoint",
            file=sys.stderr,
        )
        return 1
    manifest = {
        "version": FORMAT_VERSION,
        "status": "complete",
        "configuration": configuration,
        "configuration_sha256": configuration_sha256,
        "prime_count": len(completed),
        "first_prime": completed[0]["p"] if completed else None,
        "last_prime": completed[-1]["p"] if completed else None,
        "minimum_decimal_digits": min((r["decimal_digits"] for r in completed), default=None),
        "maximum_decimal_digits": max((r["decimal_digits"] for r in completed), default=None),
        "status_counts": counts,
        "boundary_certificate": certificate,
        "prime_quotient_inputs": [
            r["p"] for r in completed if r["status"] in {"prime_proven", "probable_prime"}
        ],
        "probable_prime_inputs": [
            r["p"] for r in completed if r["status"] == "probable_prime"
        ],
        "result_files": [f"primes/p_{r['p']}.json" for r in completed],
        "elapsed_seconds": time.monotonic() - started,
        "manifest_sha256": sha256_bytes(aggregate.encode()),
    }
    atomic_json(args.output_dir / "manifest.json", manifest)
    print(json.dumps(manifest, indent=2, sort_keys=True), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
