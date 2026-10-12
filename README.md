# GlideLapse

Native Linux timelapse grading on top of [RapidRAW](https://github.com/CyberTimon/RapidRAW):
keyframe ramping of RAW develop settings, holy-grail (day-to-night) exposure
compensation from EXIF, visual deflicker measured on rendered frames, and video
export. Nikon NEF first. Free software, AGPL-3.0.

GlideLapse edits RapidRAW's `.rrdata` sidecars and renders with RapidRAW's own
headless export, so everything you do in GlideLapse stays visible and editable
in RapidRAW.

## Status

Early development. The command-line core builds and has tests; nothing is
packaged yet. Expect breaking changes.

## Features

- [x] Scan a folder of RAW frames in capture order (EXIF time, not file name)
- [x] Holy-grail compensation from EXIF: cancels shutter / ISO / aperture jumps,
      keeps the natural drift, written as a dedicated mask layer (user exposure untouched)
- [ ] Keyframe ramping of all RapidRAW adjustments, masks and curves between keyframes
- [ ] Visual deflicker: luminance measured inside a reference area on rendered
      previews, smoothed target curve, multi-pass refinement
- [ ] Render previews and final frames through `RapidRAW export`
- [ ] Video export via ffmpeg (H.264 / H.265 / ProRes), optional motion blur
- [ ] Simple desktop UI (Tauri): folder, keyframes, reference area, run, curve view
- [ ] Linux packages (AppImage / deb)

## Roadmap

1. Command-line vertical slice: scan, holy grail, ramp, render, deflicker, re-render.
2. Deflicker quality: per-sequence calibration against RapidRAW's tone mappers,
   ignore-blacks/whites trimming, multiple reference areas.
3. Video export and a minimal UI.
4. Wider camera coverage as RapidRAW's RAW decoder grows (Nikon High Efficiency
   NEF is not decodable yet).

## How it works

```
NEF + EXIF (exiftool)  -->  keyframes, holy grail, deflicker maths  -->  .rrdata sidecars
                                                                            |
                                   previews and final frames:  RapidRAW export (headless)
                                                                            |
                                                              ffmpeg  -->  MP4 / ProRes
```

Corrections live in two reserved whole-image mask layers inside each sidecar
("GlideLapse HG" and "GlideLapse Deflicker"), added to your own exposure in EV.
Re-running GlideLapse replaces those layers; it never rewrites your edits.

## Build and run

Requirements: Rust (stable), RapidRAW 1.6.5 or later on `PATH` as `RapidRAW`,
`exiftool`, `ffmpeg`. A Vulkan-capable GPU (or Mesa's lavapipe) for RapidRAW.

```bash
cargo build --release
```

```bash
./target/release/glidelapse scan /path/to/sequence
```

```bash
./target/release/glidelapse holygrail /path/to/sequence --dry
```

Drop `--dry` to write the sidecars (a `.rrdata.bak` is kept the first time).
Close the RapidRAW GUI while GlideLapse renders; its headless export cannot run
alongside it.

## Credits

- [RapidRAW](https://github.com/CyberTimon/RapidRAW) does all the pixel work.
- Sidecar handling and ramp code adapted from
  [RapidTimelapse](https://github.com/crispin81/RapidTimelapse) (AGPL-3.0), see NOTICE.
- [ExifTool](https://exiftool.org/) and [FFmpeg](https://ffmpeg.org/).
- Workflow inspired by LRTimelapse; GlideLapse shares no code with it.

## License

AGPL-3.0-or-later. See LICENSE and NOTICE.
