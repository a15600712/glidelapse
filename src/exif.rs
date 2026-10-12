//! EXIF via the system `exiftool` (one process per call, JSON out). Gives capture order and
//! the exposure settings the holy-grail code needs.

use serde::Deserialize;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Deserialize)]
pub struct Exif {
    #[serde(rename = "SourceFile")]
    pub path: String,
    #[serde(rename = "SubSecDateTimeOriginal", default)]
    pub subsec: Option<String>,
    #[serde(rename = "DateTimeOriginal", default)]
    pub datetime: Option<String>,
    /// seconds (exiftool -n)
    #[serde(rename = "ExposureTime", default)]
    pub shutter: Option<f64>,
    #[serde(rename = "ISO", default)]
    pub iso: Option<f64>,
    #[serde(rename = "FNumber", default)]
    pub fnumber: Option<f64>,
}

impl Exif {
    /// Monotonic capture key in seconds (ignores time zone; a sequence shares one).
    pub fn capture_key(&self) -> Option<f64> {
        parse_key(self.subsec.as_deref().or(self.datetime.as_deref())?)
    }

    /// Image brightness implied by the settings, in EV: log2(t * ISO / N^2).
    /// Halving the shutter time or doubling N^2 = -1 EV; doubling ISO = +1 EV.
    pub fn brightness_ev(&self) -> Option<f64> {
        let (t, iso, n) = (self.shutter?, self.iso?, self.fnumber?);
        if !(t > 0.0 && iso > 0.0 && n > 0.0) {
            return None;
        }
        Some((t * iso / (n * n)).log2()).filter(|v| v.is_finite())
    }
}

// "2026:08:02 05:19:07.44+08:00" -> seconds on a calendar that only needs to be monotonic.
// ponytail: no real calendar maths; good enough to order one sequence, wrong across months by days.
fn parse_key(s: &str) -> Option<f64> {
    let s = s.trim();
    let mut it = s.splitn(2, ' ');
    let date = it.next()?;
    let rest = it.next()?;
    let time: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ':' || *c == '.')
        .collect();
    let mut d = date.split(':').map(|x| x.parse::<f64>().ok());
    let (y, mo, da) = (d.next()??, d.next()??, d.next()??);
    let mut t = time.split(':').map(|x| x.parse::<f64>().ok());
    let (h, mi, se) = (t.next()??, t.next()??, t.next()??);
    Some((((y * 13.0 + mo) * 32.0 + da) * 24.0 + h) * 3600.0 + mi * 60.0 + se)
}

/// Read the needed tags for `files` in one exiftool call. Order of the result follows exiftool.
pub fn read(files: &[&Path]) -> Result<Vec<Exif>, String> {
    if files.is_empty() {
        return Ok(vec![]);
    }
    let mut child = Command::new("exiftool")
        .args([
            "-j", "-n", "-q", "-fast2", "-SubSecDateTimeOriginal", "-DateTimeOriginal",
            "-ExposureTime", "-ISO", "-FNumber", "-@", "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("cannot run exiftool: {e}"))?;
    {
        let stdin = child.stdin.as_mut().ok_or("exiftool stdin")?;
        for f in files {
            let abs = f.canonicalize().map_err(|e| format!("{}: {e}", f.display()))?;
            let s = abs.to_str().ok_or_else(|| format!("{}: non-UTF-8 path", f.display()))?;
            if s.contains('\n') {
                return Err(format!("{s:?}: newline in path"));
            }
            writeln!(stdin, "{s}").map_err(|e| e.to_string())?;
        }
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("exiftool failed ({})", out.status));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("exiftool json: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_order_and_ev() {
        let a = parse_key("2026:08:02 05:19:07.44+08:00").unwrap();
        let b = parse_key("2026:08:02 05:19:08").unwrap();
        let c = parse_key("2026:08:03 00:00:00.01").unwrap();
        assert!(a < b && b < c);
        let e = Exif { path: String::new(), subsec: None, datetime: None, shutter: Some(1.0 / 125.0), iso: Some(1600.0), fnumber: Some(3.5) };
        let e2 = Exif { shutter: Some(1.0 / 250.0), ..e.clone() };
        assert!((e.brightness_ev().unwrap() - e2.brightness_ev().unwrap() - 1.0).abs() < 1e-12);
    }
}
