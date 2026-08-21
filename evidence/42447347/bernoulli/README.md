# Exact Bernoulli computation

FLINT 3.0.1 computed the exact reduced fraction

$$
B_{42{,}447{,}346}=\frac{N}{254{,}684{,}082}.
$$

The numerator has exactly 271,466,759 decimal digits and SHA-256

    d662c38e8f7e727a60079f76c9c697abbabfdb6db47ced26d80b69d15ced23d9

`run.txt` is the original computation transcript. The
`numerator_digits` field in its original JSON summary is FLINT's decimal-size
upper bound; it is one larger than the exact count. `streaming_audit.txt`
contains the exact count, recomputed numerator hash, and independent streamed
reduction modulo $p^3$.

The 259 MiB uncompressed numerator is not stored in Git. `SHA256SUMS` records
its identity alongside the committed compact evidence. Given the numerator
file, reproduce the streamed audit with:

~~~sh
scripts/audit_bernoulli_output.py \
  B_42447346.numerator.txt B_42447346.denominator.txt
~~~
