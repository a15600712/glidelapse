//! Reserved GlideLapse masks inside a RapidRAW `.rrdata` (LOG D14, RapidRAW audit sections 1-3).
//! Each correction is one mask container with a single whole-image sub-mask and an exposure
//! value; RapidRAW adds mask exposure to the global one in EV before tone mapping.

use serde_json::{json, Map, Value};

pub const HG_ID: &str = "glidelapse-hg";
pub const HG_NAME: &str = "GlideLapse HG";
pub const DEFLICKER_ID: &str = "glidelapse-deflicker";
pub const DEFLICKER_NAME: &str = "GlideLapse Deflicker";
/// RapidRAW divides the JSON `exposure` by this before `pow(2, x)`: 1 JSON unit = 1.25 EV.
pub const EXPOSURE_SCALE: f64 = 0.8;
/// RapidRAW renders at most this many mask containers (audit section 3).
pub const MAX_MASKS: usize = 32;

/// Set (or replace) the reserved mask `id` to `ev` stops. Also pins `baselineExposure: false`
/// so future RapidRAW versions do not add a camera baseline on top.
pub fn set_mask_ev(adj: &mut Map<String, Value>, id: &str, name: &str, ev: f64) -> Result<(), String> {
    if !ev.is_finite() {
        return Err(format!("{name}: non-finite EV"));
    }
    adj.insert("baselineExposure".into(), Value::Bool(false));
    let masks = adj.entry("masks").or_insert_with(|| Value::Array(vec![]));
    if !masks.is_array() {
        *masks = Value::Array(vec![]);
    }
    let list = masks.as_array_mut().unwrap();
    list.retain(|m| m.get("id").and_then(Value::as_str) != Some(id));
    if list.len() >= MAX_MASKS {
        return Err(format!("{MAX_MASKS} masks already present; RapidRAW would ignore {name}"));
    }
    list.push(json!({
        "id": id, "name": name, "visible": true, "invert": false, "opacity": 100,
        "adjustments": { "exposure": ev * EXPOSURE_SCALE },
        "subMasks": [{ "id": format!("{id}-all"), "type": "all", "visible": true,
                       "invert": false, "opacity": 100, "mode": "additive", "parameters": {} }]
    }));
    Ok(())
}

/// EV currently stored in reserved mask `id`, 0 if absent.
pub fn mask_ev(adj: &Map<String, Value>, id: &str) -> f64 {
    adj.get("masks")
        .and_then(Value::as_array)
        .and_then(|l| l.iter().find(|m| m.get("id").and_then(Value::as_str) == Some(id)))
        .and_then(|m| m.pointer("/adjustments/exposure"))
        .and_then(Value::as_f64)
        .map_or(0.0, |v| v / EXPOSURE_SCALE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_idempotent() {
        let mut adj = Map::new();
        set_mask_ev(&mut adj, HG_ID, HG_NAME, 1.0).unwrap();
        set_mask_ev(&mut adj, HG_ID, HG_NAME, -0.5).unwrap();
        set_mask_ev(&mut adj, DEFLICKER_ID, DEFLICKER_NAME, 0.25).unwrap();
        assert!(set_mask_ev(&mut adj, HG_ID, HG_NAME, f64::NAN).is_err());
        assert_eq!(adj["masks"].as_array().unwrap().len(), 2);
        assert!((mask_ev(&adj, HG_ID) + 0.5).abs() < 1e-12);
        assert_eq!(adj["masks"][0]["adjustments"]["exposure"], json!(-0.5 * 0.8));
        assert_eq!(adj["masks"][0]["subMasks"][0]["type"], "all");
        assert_eq!(adj["baselineExposure"], false);
        assert_eq!(mask_ev(&Map::new(), HG_ID), 0.0);
    }
}
