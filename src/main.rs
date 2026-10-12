//! glidelapse CLI. Commands:
//!   scan <folder>              frames in capture order with EXIF and brightness EV
//!   holygrail <folder> [--dry] compute EXIF holy-grail compensation, write the reserved HG mask

use glidelapse::{exif, holygrail, rrdata, sequence, sidecar};
use serde_json::Map;
use std::path::Path;
use std::process::exit;

struct Frame {
    seq: sequence::Frame,
    exif: exif::Exif,
}

fn load(folder: &str) -> Result<Vec<Frame>, String> {
    let frames = sequence::scan(folder)?;
    let paths: Vec<&Path> = frames.iter().map(|f| f.path.as_path()).collect();
    let mut ex = exif::read(&paths)?;
    let mut out = Vec::with_capacity(frames.len());
    for f in frames {
        let want = f.path.canonicalize().map_err(|e| e.to_string())?;
        let pos = ex
            .iter()
            .position(|e| Path::new(&e.path).canonicalize().ok().as_deref() == Some(want.as_path()))
            .ok_or_else(|| format!("no EXIF for {}", f.file))?;
        out.push(Frame { seq: f, exif: ex.swap_remove(pos) });
    }
    // capture order when every frame has a timestamp, otherwise filename order for all
    let keys: Vec<Option<f64>> = out.iter().map(|f| f.exif.capture_key()).collect();
    if keys.iter().all(Option::is_some) {
        let mut keyed: Vec<(f64, Frame)> = keys.into_iter().map(Option::unwrap).zip(out).collect();
        keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        Ok(keyed.into_iter().map(|(_, f)| f).collect())
    } else {
        eprintln!("warning: {} frame(s) without a capture time, using filename order", keys.iter().filter(|k| k.is_none()).count());
        Ok(out)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (cmd, folder) = match (args.get(1).map(String::as_str), args.get(2)) {
        (Some(c), Some(f)) => (c, f.as_str()),
        _ => {
            eprintln!("usage: glidelapse scan <folder> | holygrail <folder> [--dry]");
            exit(2)
        }
    };
    let dry = args.iter().any(|a| a == "--dry");
    if let Err(e) = run(cmd, folder, dry) {
        eprintln!("error: {e}");
        exit(1);
    }
}

fn run(cmd: &str, folder: &str, dry: bool) -> Result<(), String> {
    let frames = load(folder)?;
    let b: Vec<f64> = frames.iter().map(|f| f.exif.brightness_ev().unwrap_or(f64::NAN)).collect();
    match cmd {
        "scan" => {
            println!("idx\tfile\tcaptured\tshutter_s\tiso\tf\tbrightness_ev\tsidecar");
            for (i, f) in frames.iter().enumerate() {
                println!(
                    "{i}\t{}\t{}\t{}\t{}\t{}\t{:.4}\t{}",
                    f.seq.file,
                    f.exif.subsec.as_deref().or(f.exif.datetime.as_deref()).unwrap_or("-"),
                    f.exif.shutter.unwrap_or(f64::NAN),
                    f.exif.iso.unwrap_or(f64::NAN),
                    f.exif.fnumber.unwrap_or(f64::NAN),
                    b[i],
                    if f.seq.doc.is_some() { "yes" } else { "no" }
                );
            }
        }
        "holygrail" => {
            let missing = b.iter().filter(|v| !v.is_finite()).count();
            if missing > 0 {
                eprintln!("warning: {missing} frame(s) without exposure EXIF; jumps across them are not detected");
            }
            if frames.len() < 10 {
                eprintln!("warning: {} frames is too short for reliable jump detection", frames.len());
            }
            let h = holygrail::compensation(&b);
            println!("idx\tfile\tbrightness_ev\thg_ev");
            for (i, f) in frames.iter().enumerate() {
                println!("{i}\t{}\t{:.4}\t{:+.4}", f.seq.file, b[i], h[i]);
                if dry {
                    continue;
                }
                // re-read right before writing: RapidRAW may have rewritten the sidecar since the scan
                let mut doc = if f.seq.rrdata_path.exists() {
                    rrdata::read(&f.seq.rrdata_path)?
                } else {
                    rrdata::RrDoc { version: 1, rating: 0, adjustments: None, tags: serde_json::Value::Null, extra: Map::new() }
                };
                let mut adj = doc.adjustments.take().unwrap_or_default();
                sidecar::set_mask_ev(&mut adj, sidecar::HG_ID, sidecar::HG_NAME, h[i])
                    .map_err(|e| format!("{}: {e}", f.seq.file))?;
                doc.adjustments = Some(adj);
                rrdata::backup_if_absent(&f.seq.rrdata_path)?;
                rrdata::write(&f.seq.rrdata_path, &doc)?;
            }
            if dry {
                eprintln!("dry run: no sidecars written");
            }
        }
        _ => return Err(format!("unknown command {cmd}")),
    }
    Ok(())
}
