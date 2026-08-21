#define _POSIX_C_SOURCE 200809L

#include <errno.h>
#include <limits.h>
#include <math.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#include <flint/arith.h>
#include <flint/bernoulli.h>
#include <flint/flint.h>
#include <flint/fmpq.h>
#include <flint/fmpz.h>
#include <flint/ulong_extras.h>

#define DISCOVERY_PRIME UWORD(42447347)
#define DISCOVERY_RESIDUE "49628251800410944737487"

typedef struct
{
    atomic_bool done;
    unsigned interval_seconds;
    struct timespec started;
} heartbeat_state;

static double elapsed_seconds(const struct timespec *started)
{
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    return (double) (now.tv_sec - started->tv_sec)
        + 1.0e-9 * (double) (now.tv_nsec - started->tv_nsec);
}

static void *heartbeat_main(void *arg)
{
    heartbeat_state *state = (heartbeat_state *) arg;
    unsigned waited = 0;

    while (!atomic_load_explicit(&state->done, memory_order_relaxed))
    {
        struct timespec delay = {1, 0};
        nanosleep(&delay, NULL);
        waited++;
        if (waited >= state->interval_seconds
            && !atomic_load_explicit(&state->done, memory_order_relaxed))
        {
            fprintf(stderr, "progress: Bernoulli computation still running; elapsed %.0f s\n",
                    elapsed_seconds(&state->started));
            fflush(stderr);
            waited = 0;
        }
    }
    return NULL;
}

static void usage(FILE *stream, const char *program)
{
    fprintf(stream,
        "Usage: %s [options]\n"
        "\n"
        "Compute B_(p-1) exactly with FLINT, then independently evaluate\n"
        "p*B_(p-1) modulo p^3. With no arguments, use the reported fifth\n"
        "Lerch prime and its expected residue.\n"
        "\n"
        "Options:\n"
        "  --prime P               prime p (default: 42447347)\n"
        "  --expected R            expected p*B_(p-1) residue modulo p^3\n"
        "  --threads N             FLINT worker threads (default: 1)\n"
        "  --progress-seconds N    heartbeat interval; 0 disables (default: 60)\n"
        "  --write-prefix PATH     write PATH.numerator.txt, PATH.denominator.txt,\n"
        "                          and PATH.summary.json (existing files refused)\n"
        "  --estimate-only         print the size estimate without computing B_(p-1)\n"
        "  --help                  show this help\n",
        program);
}

static bool parse_ulong_arg(const char *text, ulong *value)
{
    char *end = NULL;
    unsigned long parsed;

    errno = 0;
    parsed = strtoul(text, &end, 10);
    if (errno != 0 || end == text || *end != '\0')
        return false;
    *value = (ulong) parsed;
    return true;
}

static bool is_decimal_integer(const char *text)
{
    const unsigned char *cursor = (const unsigned char *) text;

    if (*cursor == '+' || *cursor == '-')
        cursor++;
    if (*cursor == '\0')
        return false;
    while (*cursor >= '0' && *cursor <= '9')
        cursor++;
    return *cursor == '\0';
}

static char *suffix_path(const char *prefix, const char *suffix)
{
    size_t size = strlen(prefix) + strlen(suffix) + 1;
    char *path = malloc(size);
    if (path == NULL)
        return NULL;
    snprintf(path, size, "%s%s", prefix, suffix);
    return path;
}

static bool path_is_absent(const char *path)
{
    return access(path, F_OK) != 0 && errno == ENOENT;
}

static bool write_integer_file(const char *path, const fmpz_t value)
{
    FILE *file = fopen(path, "wx");
    bool ok;

    if (file == NULL)
        return false;
    ok = fmpz_fprint(file, value) >= 0 && fputc('\n', file) != EOF;
    if (fclose(file) != 0)
        ok = false;
    return ok;
}

int main(int argc, char **argv)
{
    ulong p = DISCOVERY_PRIME;
    ulong threads = 1;
    ulong heartbeat_seconds = 60;
    const char *expected_text = NULL;
    const char *write_prefix = NULL;
    bool prime_was_set = false;
    bool estimate_only = false;
    int exit_code = 1;
    int i;

    for (i = 1; i < argc; i++)
    {
        if (!strcmp(argv[i], "--help"))
        {
            usage(stdout, argv[0]);
            return 0;
        }
        else if (!strcmp(argv[i], "--estimate-only"))
        {
            estimate_only = true;
        }
        else if (!strcmp(argv[i], "--prime") || !strcmp(argv[i], "--threads")
                 || !strcmp(argv[i], "--progress-seconds")
                 || !strcmp(argv[i], "--expected")
                 || !strcmp(argv[i], "--write-prefix"))
        {
            const char *option = argv[i];
            if (++i >= argc)
            {
                fprintf(stderr, "error: %s requires a value\n", option);
                return 1;
            }
            if (!strcmp(option, "--prime"))
            {
                if (!parse_ulong_arg(argv[i], &p))
                {
                    fprintf(stderr, "error: invalid prime: %s\n", argv[i]);
                    return 1;
                }
                prime_was_set = true;
            }
            else if (!strcmp(option, "--threads"))
            {
                if (!parse_ulong_arg(argv[i], &threads) || threads == 0 || threads > 4096)
                {
                    fprintf(stderr, "error: invalid thread count: %s\n", argv[i]);
                    return 1;
                }
            }
            else if (!strcmp(option, "--progress-seconds"))
            {
                if (!parse_ulong_arg(argv[i], &heartbeat_seconds)
                    || heartbeat_seconds > UINT_MAX)
                {
                    fprintf(stderr, "error: invalid heartbeat interval: %s\n", argv[i]);
                    return 1;
                }
            }
            else if (!strcmp(option, "--expected"))
            {
                expected_text = argv[i];
            }
            else
            {
                write_prefix = argv[i];
            }
        }
        else
        {
            fprintf(stderr, "error: unknown option: %s\n", argv[i]);
            usage(stderr, argv[0]);
            return 1;
        }
    }

    if (!prime_was_set || p == DISCOVERY_PRIME)
    {
        if (expected_text == NULL)
            expected_text = DISCOVERY_RESIDUE;
    }

    if (expected_text != NULL && !is_decimal_integer(expected_text))
    {
        fprintf(stderr, "error: invalid expected residue: %s\n", expected_text);
        return 1;
    }

    if (p < 3 || !n_is_prime(p))
    {
        fprintf(stderr, "error: p must be an odd prime representable by FLINT's ulong\n");
        return 1;
    }

    {
        ulong n = p - 1;
        double estimated_bits = arith_bernoulli_number_size(n);
        double estimated_digits = floor(estimated_bits * 0.30102999566398119521) + 1.0;

        printf("FLINT version: %s\n", flint_version);
        printf("prime p: %lu\n", (unsigned long) p);
        printf("Bernoulli index: %lu\n", (unsigned long) n);
        printf("estimated digits in |B_n|: %.0f (upper-bound based; denominator excluded)\n",
               estimated_digits);
        printf("FLINT threads requested: %lu\n", (unsigned long) threads);
        fflush(stdout);

        if (estimate_only)
            return 0;
    }

    char *numerator_path = NULL;
    char *denominator_path = NULL;
    char *summary_path = NULL;
    if (write_prefix != NULL)
    {
        numerator_path = suffix_path(write_prefix, ".numerator.txt");
        denominator_path = suffix_path(write_prefix, ".denominator.txt");
        summary_path = suffix_path(write_prefix, ".summary.json");
        if (numerator_path == NULL || denominator_path == NULL || summary_path == NULL)
        {
            fprintf(stderr, "error: unable to allocate output paths\n");
            goto paths_done;
        }
        if (!path_is_absent(numerator_path) || !path_is_absent(denominator_path)
            || !path_is_absent(summary_path))
        {
            fprintf(stderr, "error: refusing to overwrite an existing output file for prefix %s\n",
                    write_prefix);
            goto paths_done;
        }
    }

    flint_set_num_threads((int) threads);

    fmpq_t bernoulli;
    fmpz_t denominator_without_p, modulus, p_integer, inverse, residue;
    fmpz_t expected;
    fmpq_init(bernoulli);
    fmpz_init(denominator_without_p);
    fmpz_init(modulus);
    fmpz_init(p_integer);
    fmpz_init(inverse);
    fmpz_init(residue);
    fmpz_init(expected);

    heartbeat_state heartbeat;
    pthread_t heartbeat_thread;
    bool heartbeat_started = false;
    atomic_init(&heartbeat.done, false);
    heartbeat.interval_seconds = (unsigned) heartbeat_seconds;
    clock_gettime(CLOCK_MONOTONIC, &heartbeat.started);

    if (heartbeat_seconds > 0)
    {
        if (pthread_create(&heartbeat_thread, NULL, heartbeat_main, &heartbeat) != 0)
        {
            fprintf(stderr, "error: unable to start progress heartbeat thread\n");
            goto values_done;
        }
        heartbeat_started = true;
    }

    fprintf(stderr, "computing exact B_%lu with FLINT...\n", (unsigned long) (p - 1));
    fflush(stderr);
    bernoulli_fmpq_ui(bernoulli, p - 1);

    atomic_store_explicit(&heartbeat.done, true, memory_order_relaxed);
    if (heartbeat_started)
    {
        pthread_join(heartbeat_thread, NULL);
        heartbeat_started = false;
    }

    double elapsed = elapsed_seconds(&heartbeat.started);
    printf("computation elapsed seconds: %.3f\n", elapsed);
    printf("numerator digit upper bound: %zu\n",
           fmpz_sizeinbase(fmpq_numref(bernoulli), 10));
    printf("denominator digit upper bound: %zu\n",
           fmpz_sizeinbase(fmpq_denref(bernoulli), 10));

    if (!fmpq_is_canonical(bernoulli))
    {
        fprintf(stderr, "error: FLINT returned a noncanonical fraction\n");
        goto values_done;
    }
    fmpz_set_ui(p_integer, p);
    if (!fmpz_divisible(fmpq_denref(bernoulli), p_integer))
    {
        fprintf(stderr, "error: denominator is not divisible by p\n");
        goto values_done;
    }

    fmpz_divexact_ui(denominator_without_p, fmpq_denref(bernoulli), p);
    if (fmpz_divisible(denominator_without_p, p_integer))
    {
        fprintf(stderr, "error: denominator is divisible by p more than once\n");
        goto values_done;
    }

    fmpz_pow_ui(modulus, p_integer, 3);
    if (!fmpz_invmod(inverse, denominator_without_p, modulus))
    {
        fprintf(stderr, "error: D/p is not invertible modulo p^3\n");
        goto values_done;
    }
    fmpz_mul(residue, fmpq_numref(bernoulli), inverse);
    fmpz_mod(residue, residue, modulus);

    printf("modulus p^3: ");
    fmpz_print(modulus);
    printf("\ncomputed p*B_(p-1) mod p^3: ");
    fmpz_print(residue);
    printf("\n");

    bool matched = true;
    if (expected_text != NULL)
    {
        if (fmpz_set_str(expected, expected_text, 10) != 0)
        {
            fprintf(stderr, "error: invalid expected residue: %s\n", expected_text);
            goto values_done;
        }
        fmpz_mod(expected, expected, modulus);
        matched = fmpz_equal(residue, expected);
        printf("expected residue: %s\n", expected_text);
        printf("expected residue match: %s\n", matched ? "YES" : "NO");
    }
    fflush(stdout);

    if (write_prefix != NULL)
    {
        if (!write_integer_file(numerator_path, fmpq_numref(bernoulli))
            || !write_integer_file(denominator_path, fmpq_denref(bernoulli)))
        {
            fprintf(stderr, "error: failed while writing exact fraction files\n");
            goto values_done;
        }

        char *modulus_text = fmpz_get_str(NULL, 10, modulus);
        char *residue_text = fmpz_get_str(NULL, 10, residue);
        FILE *summary = fopen(summary_path, "wx");
        if (modulus_text == NULL || residue_text == NULL || summary == NULL)
        {
            fprintf(stderr, "error: unable to create summary file\n");
            if (summary != NULL)
                fclose(summary);
            flint_free(modulus_text);
            flint_free(residue_text);
            goto values_done;
        }
        fprintf(summary,
            "{\n"
            "  \"flint_version\": \"%s\",\n"
            "  \"prime\": %lu,\n"
            "  \"bernoulli_index\": %lu,\n"
            "  \"threads_requested\": %lu,\n"
            "  \"elapsed_seconds\": %.6f,\n"
            "  \"numerator_digits_upper_bound\": %zu,\n"
            "  \"denominator_digits_upper_bound\": %zu,\n"
            "  \"modulus_p_cubed\": \"%s\",\n"
            "  \"p_times_bernoulli_mod_p_cubed\": \"%s\",\n",
            flint_version, (unsigned long) p, (unsigned long) (p - 1),
            (unsigned long) threads, elapsed,
            fmpz_sizeinbase(fmpq_numref(bernoulli), 10),
            fmpz_sizeinbase(fmpq_denref(bernoulli), 10),
            modulus_text, residue_text);
        if (expected_text == NULL)
        {
            fprintf(summary,
                "  \"expected_residue\": null,\n"
                "  \"expected_residue_match\": null\n");
        }
        else
        {
            fprintf(summary,
                "  \"expected_residue\": \"%s\",\n"
                "  \"expected_residue_match\": %s\n",
                expected_text, matched ? "true" : "false");
        }
        fprintf(summary, "}\n");
        if (fclose(summary) != 0)
        {
            fprintf(stderr, "error: failed while closing summary file\n");
            flint_free(modulus_text);
            flint_free(residue_text);
            goto values_done;
        }
        flint_free(modulus_text);
        flint_free(residue_text);
        printf("wrote exact numerator: %s\n", numerator_path);
        printf("wrote exact denominator: %s\n", denominator_path);
        printf("wrote summary: %s\n", summary_path);
    }

    exit_code = matched ? 0 : 3;

values_done:
    if (heartbeat_started)
    {
        atomic_store_explicit(&heartbeat.done, true, memory_order_relaxed);
        pthread_join(heartbeat_thread, NULL);
    }
    fmpz_clear(expected);
    fmpz_clear(residue);
    fmpz_clear(inverse);
    fmpz_clear(p_integer);
    fmpz_clear(modulus);
    fmpz_clear(denominator_without_p);
    fmpq_clear(bernoulli);
    flint_cleanup_master();

paths_done:
    free(summary_path);
    free(denominator_path);
    free(numerator_path);
    return exit_code;
}
