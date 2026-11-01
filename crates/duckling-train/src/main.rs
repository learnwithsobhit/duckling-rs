//! Classifier training / generation CLI.
//!
//! Ports `exe/Duckling/Ranking/Train.hs` + `Generate.hs`.
//!
//! Float tolerance (IEEE-754 `ln` / `/` of identical integer counts):
//!   --abs-tol 1e-12
//!   --rel-tol 1e-9
//! A finite pair matches when `|a-b| <= max(abs, rel * max(|a|,|b|))`.
//! Infinities must match exactly. Counts (`n`) and feature keys must be identical.

use std::path::PathBuf;
use std::process::ExitCode;

use duckling::locale::Locale;
use duckling_train::compare::{compare, Tol, DEFAULT_ABS_TOL, DEFAULT_REL_TOL};
use duckling_train::corpus::{exported_locales, load_corpus};
use duckling_train::dataset::train_locale;
use duckling_train::json_io::{classifiers_to_json, load_classifiers_file, write_classifiers};
use duckling_train::nb::Classifiers;

fn usage() -> &'static str {
    "\
duckling-train — Naive Bayes classifier trainer (Haskell Train.hs / Generate.hs)

USAGE:
  duckling-train verify [--locale LOC] [--exported DIR] [--out DIR]
                        [--abs-tol F] [--rel-tol F] [--show N]
  duckling-train train   --locale LOC [--out FILE]
  duckling-train dump    --locale LOC [--out FILE]
  duckling-train compare --trained FILE --exported FILE
                        [--abs-tol F] [--rel-tol F] [--show N]

LOC is e.g. EN_XX, EN_GB, FR_XX (same names as data/classifiers/*.json).

verify  trains from the Time corpora and compares against the exported
        tables. Exit 0 iff every locale matches within float tolerance.
train   trains one locale and prints a summary (writes JSON if --out).
dump    trains one locale and writes ExportClassifiers-format JSON.
compare diffs two already-written JSON files.

Float tolerance (default abs=1e-12, rel=1e-9):
  finite values match when |a-b| <= max(abs, rel * max(|a|,|b|));
  infinities must be equal; `n` and feature keys must be identical.
"
}

struct Args {
    cmd: String,
    locale: Option<String>,
    exported: Option<PathBuf>,
    trained: Option<PathBuf>,
    out: Option<PathBuf>,
    abs_tol: f64,
    rel_tol: f64,
    show: usize,
}

fn parse_args() -> Result<Args, String> {
    let mut argv = std::env::args().skip(1);
    let cmd = argv.next().ok_or_else(|| usage().to_string())?;
    if cmd == "-h" || cmd == "--help" || cmd == "help" {
        return Err(usage().to_string());
    }
    let mut a = Args {
        cmd,
        locale: None,
        exported: None,
        trained: None,
        out: None,
        abs_tol: DEFAULT_ABS_TOL,
        rel_tol: DEFAULT_REL_TOL,
        show: 20,
    };
    while let Some(x) = argv.next() {
        match x.as_str() {
            "--locale" => a.locale = Some(argv.next().ok_or("--locale needs a value")?),
            "--exported" => a.exported = Some(PathBuf::from(argv.next().ok_or("--exported needs a path")?)),
            "--trained" => a.trained = Some(PathBuf::from(argv.next().ok_or("--trained needs a path")?)),
            "--out" => a.out = Some(PathBuf::from(argv.next().ok_or("--out needs a path")?)),
            "--abs-tol" => a.abs_tol = argv.next().ok_or("--abs-tol")?.parse().map_err(|e| format!("abs-tol: {e}"))?,
            "--rel-tol" => a.rel_tol = argv.next().ok_or("--rel-tol")?.parse().map_err(|e| format!("rel-tol: {e}"))?,
            "--show" => a.show = argv.next().ok_or("--show")?.parse().map_err(|e| format!("show: {e}"))?,
            "-h" | "--help" => return Err(usage().to_string()),
            other => return Err(format!("unknown argument {other}\n{}", usage())),
        }
    }
    Ok(a)
}

fn parse_locale(s: &str) -> Result<Locale, String> {
    let up = s.trim().to_uppercase();
    let mut parts = up.split('_');
    let lang = parts.next().and_then(duckling::Lang::parse).ok_or_else(|| format!("bad locale {s}"))?;
    let region = match parts.next() {
        None | Some("XX") => None,
        Some(r) => Some(duckling::Region::parse(r).ok_or_else(|| format!("bad region {r}"))?),
    };
    if parts.next().is_some() {
        return Err(format!("bad locale {s}"));
    }
    Ok(Locale { lang, region })
}

fn default_exported_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../duckling/data/classifiers")
}

fn table_name(loc: Locale) -> String {
    loc.to_string()
}

fn train_one(loc: Locale) -> Result<(Classifiers, usize), String> {
    let corpus = load_corpus(loc)?;
    let n = corpus.examples.len();
    Ok((train_locale(&corpus), n))
}

fn print_diff(name: &str, d: &duckling_train::compare::Diff, show: usize) {
    if d.ok() {
        println!("{name}: OK  max|Δ|={:.3e}", d.max_abs_delta);
        return;
    }
    println!(
        "{name}: FAIL  missing={} extra={} field={} max|Δ|={:.3e}",
        d.missing_rules.len(),
        d.extra_rules.len(),
        d.field.len(),
        d.max_abs_delta
    );
    for r in d.missing_rules.iter().take(show) {
        println!("  missing rule {r:?}");
    }
    for r in d.extra_rules.iter().take(show) {
        println!("  extra rule {r:?}");
    }
    for r in d.field.iter().take(show) {
        println!("  {r}");
    }
    let hidden = d.missing_rules.len().saturating_add(d.extra_rules.len()).saturating_add(d.field.len()).saturating_sub(show * 3);
    if hidden > 0 {
        println!("  … {hidden} more (raise --show)");
    }
}

fn run() -> Result<ExitCode, String> {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return Ok(if e.contains("USAGE") { ExitCode::SUCCESS } else { ExitCode::from(2) });
        }
    };
    let tol = Tol { abs: args.abs_tol, rel: args.rel_tol };

    match args.cmd.as_str() {
        "train" => {
            let loc = parse_locale(args.locale.as_deref().ok_or("train requires --locale")?)?;
            let (cs, n_ex) = train_one(loc)?;
            println!("{}: {} examples, {} rules", table_name(loc), n_ex, cs.len());
            if let Some(out) = args.out {
                write_classifiers(&out, &cs)?;
                println!("wrote {}", out.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        "dump" => {
            let loc = parse_locale(args.locale.as_deref().ok_or("dump requires --locale")?)?;
            let (cs, n_ex) = train_one(loc)?;
            match args.out {
                Some(out) => {
                    write_classifiers(&out, &cs)?;
                    eprintln!("{}: {} examples, {} rules → {}", table_name(loc), n_ex, cs.len(), out.display());
                }
                None => print!("{}", classifiers_to_json(&cs)),
            }
            Ok(ExitCode::SUCCESS)
        }
        "compare" => {
            let trained = args.trained.ok_or("compare requires --trained")?;
            let exported = args.exported.ok_or("compare requires --exported")?;
            let t = load_classifiers_file(&trained)?;
            let e = load_classifiers_file(&exported)?;
            let d = compare(&t, &e, tol);
            print_diff(&format!("{} vs {}", trained.display(), exported.display()), &d, args.show);
            Ok(if d.ok() { ExitCode::SUCCESS } else { ExitCode::from(1) })
        }
        "verify" => {
            let exported_dir = args.exported.unwrap_or_else(default_exported_dir);
            let locales: Vec<Locale> = match args.locale {
                Some(s) => vec![parse_locale(&s)?],
                None => exported_locales(),
            };
            let mut failed = 0usize;
            let mut checked = 0usize;
            for loc in locales {
                let name = table_name(loc);
                let path = exported_dir.join(format!("{name}.json"));
                if !path.exists() {
                    println!("{name}: SKIP (no {})", path.display());
                    continue;
                }
                let (cs, n_ex) = train_one(loc)?;
                let exported = load_classifiers_file(&path)?;
                let d = compare(&cs, &exported, tol);
                print_diff(&format!("{name} ({n_ex} ex, {} rules)", cs.len()), &d, args.show);
                if let Some(dir) = &args.out {
                    write_classifiers(&dir.join(format!("{name}.json")), &cs)?;
                }
                checked += 1;
                if !d.ok() {
                    failed += 1;
                }
            }
            println!(
                "verify: {checked} locales, {failed} failed  (abs-tol={} rel-tol={})",
                tol.abs, tol.rel
            );
            Ok(if failed == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) })
        }
        other => Err(format!("unknown command {other}\n{}", usage())),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}
