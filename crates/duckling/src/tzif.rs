//! Reader for compiled Olson (`TZif`) time zone files, mirroring `timezone-olson`'s
//! `getTimeZoneSeriesFromOlsonFile` so that the server resolves zones the way upstream does.

use crate::time_zone::{TimeZone, TimeZoneSeries, UtcTime};

/// One `ttinfo` record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TtInfo {
    pub utoff: i32,
    pub is_dst: bool,
    pub abbr: String,
}

/// Raw contents of the (64-bit, when present) data block.
#[derive(Clone, Debug)]
pub struct RawTzif {
    pub transitions: Vec<(i64, usize)>,
    pub ttinfos: Vec<TtInfo>,
}

struct Reader<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.i + n > self.b.len() {
            return Err("truncated TZif data".into());
        }
        let s = &self.b[self.i..self.i + n];
        self.i += n;
        Ok(s)
    }
    fn be32(&mut self) -> Result<i32, String> {
        Ok(i32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn be64(&mut self) -> Result<i64, String> {
        Ok(i64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }
}

struct Header {
    version: u8,
    isutcnt: usize,
    isstdcnt: usize,
    leapcnt: usize,
    timecnt: usize,
    typecnt: usize,
    charcnt: usize,
}

fn header(r: &mut Reader) -> Result<Header, String> {
    if r.take(4)? != b"TZif" {
        return Err("not a TZif file".into());
    }
    let version = r.take(1)?[0];
    r.take(15)?;
    let mut n = [0usize; 6];
    for x in n.iter_mut() {
        *x = r.be32()? as usize;
    }
    Ok(Header {
        version,
        isutcnt: n[0],
        isstdcnt: n[1],
        leapcnt: n[2],
        timecnt: n[3],
        typecnt: n[4],
        charcnt: n[5],
    })
}

fn block(r: &mut Reader, h: &Header, wide: bool) -> Result<RawTzif, String> {
    let mut times = Vec::with_capacity(h.timecnt);
    for _ in 0..h.timecnt {
        times.push(if wide { r.be64()? } else { r.be32()? as i64 });
    }
    let idxs: Vec<usize> = r.take(h.timecnt)?.iter().map(|&x| x as usize).collect();
    let mut raw = Vec::with_capacity(h.typecnt);
    for _ in 0..h.typecnt {
        let off = r.be32()?;
        let b = r.take(2)?;
        raw.push((off, b[0] != 0, b[1] as usize));
    }
    let chars = r.take(h.charcnt)?;
    r.take(h.leapcnt * if wide { 12 } else { 8 })?;
    r.take(h.isstdcnt)?;
    r.take(h.isutcnt)?;
    let ttinfos = raw
        .into_iter()
        .map(|(utoff, is_dst, ai)| {
            let end = chars[ai.min(chars.len())..].iter().position(|&c| c == 0).map(|p| ai + p).unwrap_or(chars.len());
            TtInfo { utoff, is_dst, abbr: String::from_utf8_lossy(&chars[ai.min(chars.len())..end]).into_owned() }
        })
        .collect();
    let mut transitions = Vec::with_capacity(h.timecnt);
    for (t, i) in times.into_iter().zip(idxs) {
        if i >= h.typecnt {
            return Err("transition refers to a missing ttinfo".into());
        }
        transitions.push((t, i));
    }
    Ok(RawTzif { transitions, ttinfos })
}

/// Parse a TZif file: the 32-bit block and, for version 2+, the 64-bit block.
pub fn parse_blocks(bytes: &[u8]) -> Result<(RawTzif, Option<RawTzif>), String> {
    let mut r = Reader { b: bytes, i: 0 };
    let h1 = header(&mut r)?;
    let v1 = block(&mut r, &h1, false)?;
    if h1.version < b'2' {
        return Ok((v1, None));
    }
    let h2 = header(&mut r)?;
    let v2 = block(&mut r, &h2, true)?;
    Ok((v1, Some(v2)))
}

fn to_zone(t: &TtInfo) -> TimeZone {
    // Whole minutes, rounded to nearest (as `timezone-olson` does).
    TimeZone { offset_minutes: (t.utoff as f64 / 60.0).round_ties_even() as i32, summer_only: t.is_dst, name: t.abbr.clone() }
}
fn to_utc(secs: i64) -> UtcTime {
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    UtcTime { mjd: crate::time_zone::UNIX_EPOCH_MJD + days, psecs: sod * 1_000_000_000_000 }
}

/// `getTimeZoneSeriesFromOlsonFile`. Upstream merges the transitions of the 32-bit and 64-bit
/// blocks and takes the initial zone from the 32-bit block's first `ttinfo`; this reproduces
/// that (checked against `timezone-olson` output for a sample of zones).
pub fn parse_series(bytes: &[u8]) -> Result<TimeZoneSeries, String> {
    let (v1, v2) = parse_blocks(bytes)?;
    if v1.ttinfos.is_empty() {
        return Err("TZif file without ttinfo records".into());
    }
    let default_zone = to_zone(&v1.ttinfos[0]);
    let mut all: Vec<(i64, TimeZone)> = Vec::new();
    for (t, i) in &v1.transitions {
        all.push((*t, to_zone(&v1.ttinfos[*i])));
    }
    if let Some(v2) = &v2 {
        for (t, i) in &v2.transitions {
            all.push((*t, to_zone(&v2.ttinfos[*i])));
        }
    }
    all.sort_by_key(|(t, _)| *t);
    all.dedup_by_key(|(t, _)| *t);
    let transitions = all.into_iter().rev().map(|(t, z)| (to_utc(t), z)).collect();
    Ok(TimeZoneSeries { default_zone, transitions })
}
