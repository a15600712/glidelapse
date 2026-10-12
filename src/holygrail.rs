//! Holy-grail compensation from EXIF (LOG F37): cancel the exposure-setting jumps, keep the
//! smooth drift. Input: per-frame image brightness in EV (`exif::Exif::brightness_ev`), in
//! capture order. Output: EV to add per frame (H), written to the reserved HG mask.

pub const MIN_JUMP_EV: f64 = 0.10;
pub const MAD_K: f64 = 6.0 * 1.4826;
pub const WINDOW: usize = 4;

/// `brightness[i]` may be NaN for frames without EXIF; they inherit the previous H.
pub fn compensation(brightness: &[f64]) -> Vec<f64> {
    let n = brightness.len();
    let d: Vec<f64> = (0..n)
        .map(|i| if i == 0 { 0.0 } else { brightness[i] - brightness[i - 1] })
        .collect();
    let mut h = vec![0.0; n];
    for i in 1..n {
        let lo = i.saturating_sub(WINDOW).max(1);
        let hi = (i + WINDOW).min(n - 1);
        let neigh: Vec<f64> = (lo..=hi).filter(|&j| j != i && d[j].is_finite()).map(|j| d[j]).collect();
        let drift = median(&neigh).unwrap_or(0.0);
        let devs: Vec<f64> = neigh.iter().map(|x| (x - drift).abs()).collect();
        let thresh = MIN_JUMP_EV.max(MAD_K * median(&devs).unwrap_or(0.0));
        let jump = if d[i].is_finite() && (d[i] - drift).abs() >= thresh { d[i] - drift } else { 0.0 };
        h[i] = h[i - 1] - jump;
    }
    h
}

fn median(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = s.len() / 2;
    Some(if s.len() % 2 == 0 { (s[m - 1] + s[m]) / 2.0 } else { s[m] })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F37 fixture: 160 frames, shutter drift -0.005 EV/frame, shutter halved at 40,
    /// ISO doubled at 80, aperture x sqrt2 at 120 -> brightness jumps -1, +1, -1 -> H = +1, 0, +1.
    #[test]
    fn synthetic_exif() {
        let mut t: f64 = 1.0 / 30.0;
        let (mut iso, mut n2) = (100.0f64, 8.0f64 * 8.0);
        let mut b = Vec::new();
        for i in 0..160 {
            if i == 40 { t /= 2.0; }
            if i == 80 { iso *= 2.0; }
            if i == 120 { n2 *= 2.0; }
            b.push((t * iso / n2).log2());
            t *= 2f64.powf(-0.005);
        }
        let h = compensation(&b);
        for (i, v) in h.iter().enumerate() {
            let want = match i { 0..=39 => 0.0, 40..=79 => 1.0, 80..=119 => 0.0, _ => 1.0 };
            assert!((v - want).abs() < 1e-9, "frame {i}: got {v}, want {want}");
        }
        // corrected brightness keeps the drift only
        let slope = (b[159] + h[159] - (b[0] + h[0])) / 159.0;
        assert!((slope + 0.005).abs() < 1e-9, "slope {slope}");
        assert_eq!(compensation(&h.iter().map(|_| 0.0).collect::<Vec<_>>()), vec![0.0; 160]);
    }
}
