# GlideLapse — agent notes

Native-Linux timelapse grading on top of RapidRAW: keyframe ramping, EXIF
holy-grail compensation, visual deflicker, video export. Rust core (lib + CLI),
later a plain Tauri UI. AGPL-3.0. Nikon NEF first.

Rules
- Clean-room: GlideLapse shares no code with LRTimelapse; never add vendor
  packages, decompiled output or proprietary files to this repo.
- Reuse before writing: RapidRAW renders (`RapidRAW export`), exiftool reads
  metadata, ffmpeg encodes. Keep the code minimal; no frameworks for one use.
- Privacy: no hostnames, IPs, logins, e-mails, paths or tokens in the repo.
  Install the guard once per clone: `ln -sf ../../tools/pre-commit-privacy.sh .git/hooks/pre-commit`.
- Public tree stays a normal open-source project: README, code, licence. No
  logs or audit detail here.

Day-to-day development happens in the private repo `glidelapse-dev` (remote
`dev`); this public repo receives polished states only. Maintainer context lives
in the private repo `glidelapse-notes` (same owner):
`HANDOFF.md` first, then `LOG.md` "Next steps", then `CLAUDE-full.md`. If it is
checked out locally it sits at `notes/` (git-ignored).

Build: `cargo build --release`; tests: `cargo test`. Rendering needs RapidRAW
1.6.5+ on PATH and a Vulkan device; without them only the unit tests run.
