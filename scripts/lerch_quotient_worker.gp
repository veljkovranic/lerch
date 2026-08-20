\\ Exact Lerch-quotient construction and compositeness screening.
\\
\\ This file is loaded by search_prime_lerch_quotients.py.  Keep stdout quiet
\\ except for the single LQRESULT line emitted by lq_search_one().

lq_search_one(p, trial_bound, inline_digits, candidate_path) =
{
  my(t, numerator, ell, digits, build_ms, screen_ms);
  my(trial_product, common, factor_found = 0, fermat_residue);
  my(status = "", evidence = 0, probable = 0, inline_value = "");
  my(mod_p, mod_1, mod_2, mod_3);

  if(!isprime(p) || p < 3 || p % 2 == 0,
    error("p must be an odd prime")
  );

  t = gettime();
  numerator = sum(k = 1, p - 1, k^(p - 1)) - p - (p - 1)!;
  if(numerator % p^2 != 0,
    error("Lerch numerator is not divisible by p^2")
  );
  ell = numerator / p^2;
  digits = if(ell == 0, 1, #Str(abs(ell)));
  mod_p = ell % p;
  mod_1 = ell % 1000000007;
  mod_2 = ell % 1000000009;
  mod_3 = ell % 2147483647;
  if(digits <= inline_digits, inline_value = Str(ell));
  build_ms = gettime();

  if(ell == 0,
    status = "nonprime_zero",
    if(ell == 1,
      status = "nonprime_one",
      trial_product = 1;
      forprime(q = 2, trial_bound, trial_product *= q);
      common = gcd(ell, trial_product);
      if(common > 1 && common < ell,
        factor_found = factor(common)[1, 1];
        status = "composite_factor";
        evidence = factor_found,
        if(common == ell && isprime(ell),
          status = "prime_proven";
          evidence = ell,
          fermat_residue = lift(Mod(2, ell)^(ell - 1));
          if(fermat_residue != 1,
            status = "composite_fermat";
            evidence = 2,
            if(ispseudoprime(ell),
              status = "probable_prime";
              probable = 1,
              status = "composite_bpsw";
              evidence = 0
            )
          )
        )
      )
    )
  );
  screen_ms = gettime();

  if(probable,
    write(candidate_path, ell)
  );

  print("LQRESULT|", p, "|", digits, "|", build_ms, "|", screen_ms,
        "|", status, "|", evidence, "|", mod_p, "|", mod_1, "|",
        mod_2, "|", mod_3, "|", inline_value);
}
