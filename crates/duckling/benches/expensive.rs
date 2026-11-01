//! Criterion benches that replay the two cases in upstream `exe/DucklingExpensive.hs`.
//!
//! Haskell `duckling-expensive N` loops `debug` (analyze + print parse trees) N times
//! on both sentences. These benches time `parse` (analyze + format, no I/O) so the
//! number is the engine cost. Compare with:
//!
//! ```text
//! cargo bench -p duckling --bench expensive
//! ./tools/oracle/bench_expensive.sh 50
//! ```

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use duckling::dims::time::calendar::from_gregorian;
use duckling::time_zone::{DucklingTime, TimeZoneSeries, UtcTime};
use duckling::{parse, Context, Dim, Lang, Locale, Options};

/// Same sentences as `vendor/duckling/exe/DucklingExpensive.hs`.
const EN: &str = "Monday 3rd at 9.30am or 2.30pm, Saturday 8th at 10.30am, Tuesday 11th at 2pm, Wednesday 12th at 2.30pm, Friday 14th at 12.30pm xx";
/// UTF-8 for the Haskell string that uses `S\195\161bado`.
const ES: &str = "Horario es de Lunes a Viernes de 2 pm a 10 pm. Sábado de 9 am a 7 pm";

/// `Duckling.Testing.Types.testContext` with a different locale:
/// Tuesday 2013-02-12 04:30 UTC, zone series UTC−2.
fn test_context(lang: Lang) -> Context {
    let mjd = from_gregorian(2013, 2, 12);
    let psecs = (4 * 3600 + 30 * 60) * 1_000_000_000_000i64;
    Context {
        reference_time: DucklingTime {
            utc: UtcTime { mjd, psecs },
            series: TimeZoneSeries::fixed(-120),
        },
        locale: Locale::new(lang, None),
    }
}

fn parse_en(ctx: &Context, opts: &Options) -> usize {
    parse(EN, ctx, opts, &[Dim::Time]).len()
}

fn parse_es(ctx: &Context, opts: &Options) -> usize {
    parse(ES, ctx, opts, &[Dim::Time]).len()
}

fn duckling_expensive(c: &mut Criterion) {
    let ctx_en = test_context(Lang::EN);
    let ctx_es = test_context(Lang::ES);
    let opts = Options { with_latent: false };
    // Force rule/regex compilation out of the timed region.
    let _ = parse_en(&ctx_en, &opts);
    let _ = parse_es(&ctx_es, &opts);

    let mut g = c.benchmark_group("duckling_expensive");
    g.bench_function("en_time", |b| {
        b.iter(|| black_box(parse_en(&ctx_en, &opts)));
    });
    g.bench_function("es_time", |b| {
        b.iter(|| black_box(parse_es(&ctx_es, &opts)));
    });
    // One iteration of the Haskell `replicateM` body (both sentences, no debug print).
    g.bench_function("both", |b| {
        b.iter(|| {
            let a = parse_en(&ctx_en, &opts);
            let c = parse_es(&ctx_es, &opts);
            black_box(a + c)
        });
    });
    g.finish();
}

criterion_group!(benches, duckling_expensive);
criterion_main!(benches);
