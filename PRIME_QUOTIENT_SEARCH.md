# Prime-valued Lerch quotient search

This search is separate from the modular Lerch-prime search.  It constructs
the full integer

\[
\ell_p = \frac{\sum_{a=1}^{p-1}a^{p-1}-p-(p-1)!}{p^2}
\]

for every odd prime in an inclusive range and determines whether it is
composite or a probable prime.

## Why `p <= 100003`

The number of decimal digits is asymptotic to `p*log10(p)`.  A 500,000-digit
Lerch quotient occurs near `p = 100003`.  The production script includes that
prime as a boundary sentinel and records the exact digit count, so the final
manifest shows whether the endpoint lies just below or above 500,000 digits.
It also records the directly checked lower-bound inequality
`((p-1)^(p-1) - (p-1)! - p)/p^2 >= 10^500000` at the endpoint.
The full run starts at `p = 3`; it therefore reproduces the old range instead
of relying only on the published 300,000-digit statement.

## Method and evidence

PARI/GP constructs each quotient exactly and checks that the numerator is
divisible by `p^2`.  It then:

1. computes a GCD with the product of all primes through 10,000;
2. records a proper prime factor when that GCD is nontrivial;
3. otherwise applies a base-2 Fermat compositeness test;
4. applies PARI's `ispseudoprime` test only if base 2 passes;
5. retains the complete decimal quotient if it remains a probable prime.

A proper factor or a failed Fermat test proves compositeness.  A
`probable_prime` result is deliberately not presented as a proof and must be
checked independently.  Three fixed-modulus fingerprints and `ell_p mod p`
are retained for every exact quotient.

Each `primes/p_P.json` file is written atomically and includes a configuration
hash and result hash.  Resume accepts only files whose hashes and configuration
match.  `manifest.json` is written only after the full requested prime list is
complete.

## Server run

Install PARI/GP once:

~~~sh
sudo apt-get update
sudo apt-get install -y pari-gp
~~~

Launch on the 64-core EPYC:

~~~sh
cd ~/lerch_prime_search
nohup scripts/search_prime_quotients_500k.sh 64 \
  > search-prime-quotients-500k.log 2>&1 &
echo $! > search-prime-quotients-500k.pid
tail -f search-prime-quotients-500k.log
~~~

Stopping the parent process preserves completed per-prime files.  Run the same
launch command again to resume.

Live status:

~~~sh
watch -n 30 '
R=results/prime-lerch-quotients-3-100003
n=$(find "$R/primes" -type f -name "p_*.json" 2>/dev/null | wc -l)
echo "$n / 9592 odd-prime inputs complete"
find "$R/primes" -type f -name "p_*.json" -print0 2>/dev/null |
  xargs -0 -r jq -r "select(.status == \"probable_prime\" or .status == \"prime_proven\") | [.p,.decimal_digits,.status] | @tsv"
'
~~~

The expected displayed prime-valued case is `p=5`, where `ell_5=13`.  Any
additional line, especially `probable_prime`, requires immediate independent
verification.

After completion, audit coverage, hashes, retained candidates, and totals:

~~~sh
scripts/audit_prime_lerch_quotients.py \
  results/prime-lerch-quotients-3-100003
~~~

## Smoke test

~~~sh
tmp=$(mktemp -d)
python3 scripts/search_prime_lerch_quotients.py \
  --start-prime 3 --end-prime 100 --workers 2 --output-dir "$tmp"
jq '.prime_quotient_inputs, .status_counts' "$tmp/manifest.json"
~~~
