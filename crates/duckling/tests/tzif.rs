//! The TZif reader must produce exactly the series `timezone-olson` builds from the same files
//! (expected dumps made with `tools/oracle/Tz.hs`).

use std::fs;
use std::path::PathBuf;

use duckling::time_zone::TimeZone;

fn tz(z: &TimeZone) -> String {
    format!("{},{},{}", z.offset_minutes, if z.summer_only { "True" } else { "False" }, z.name)
}

#[test]
fn matches_timezone_olson() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/tz");
    let zones = [
        "America/Los_Angeles",
        "Asia/Kolkata",
        "Europe/London",
        "UTC",
        "Asia/Tokyo",
        "Africa/Casablanca",
        "America/Sao_Paulo",
        "America/St_Johns",
        "Australia/Lord_Howe",
        "Europe/Dublin",
        "Pacific/Auckland",
    ];
    for z in zones {
        let s = duckling::tzif::parse_series(&fs::read(root.join("tzdata").join(z)).unwrap()).unwrap();
        let mut out = format!("default {}\nn {}\n", tz(&s.default_zone), s.transitions.len());
        for (t, zz) in &s.transitions {
            out += &format!("{} {} {}\n", t.mjd, t.psecs, tz(zz));
        }
        let expected = fs::read_to_string(root.join("tzexpected").join(format!("{z}.txt"))).unwrap();
        assert_eq!(out, expected, "zone {z}");
    }
}
