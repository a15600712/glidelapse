//! GlideLapse core: sequence scan, EXIF, holy-grail compensation, reserved-mask sidecar writes.
//! Rendering is delegated to `RapidRAW export` (see docs/audit/rapidraw-1.6.5.md).

pub mod vendor {
    pub mod interpolate;
    pub mod params;
    pub mod rrdata;
    pub mod sequence;
}
pub use vendor::{interpolate, params, rrdata, sequence};

pub mod exif;
pub mod holygrail;
pub mod sidecar;
