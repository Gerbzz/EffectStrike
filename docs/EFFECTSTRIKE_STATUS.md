# EffectStrike integration and verification ledger

## Source and legal provenance

| Item | Status |
| --- | --- |
| Foundation | Full [`storytold/effectcraft`](https://github.com/storytold/effectcraft) Rust application source, pinned at `0311bdc34d0dc10c2d18ff7875b4f52d483e5e27` (15 October 2026). |
| Source license | MIT OR Apache-2.0; original `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, and third-party `ATTRIBUTION.md` retained and extended. |
| Trademark separation | Upstream `docs/brand/` ArtCraft marks removed from the public snapshot. Product is independent of ArtCraft. |
| Current approved mark | PWDS company PNG copied unchanged from `Gerbzz/PWDS-Strike-Launcher`, verified against Git blob `e6478d3dff4406c57679126e63c40282774fe843`; separate brand terms and sidecar. |
| Build route | Rust 1.95+, Cargo package `effectcraft`; internal names and `.ecproj` retained for format compatibility. |

The published root commit deliberately omits upstream history so the removed trademark image files cannot be retrieved from this repository's commit history. The exact upstream commit and copyright are recorded above and in `NOTICE`.

## Functional coverage

“Present” means inspected in the pinned upstream source and its architecture/documentation; it does not mean tested in this fork.

| Workflow | Source presence | Fork test |
| --- | --- | --- |
| Create and edit a composition with layers | Present: project model, engine commands, egui timeline/viewer | Not run |
| Import footage and save/reopen `.ecproj` | Present: media, project serialization, UI and CLI commands | Not run |
| Animate with keyframes and graph editor | Present: keyframe crate and UI panels | Not run |
| Apply effects and expressions | Present: effects registry, expression engine, controls | Not run |
| Use cameras, lights, and 3D layers | Present: model/render crates and viewer controls | Not run |
| Queue and export a render | Present: export/render queue and CLI | Not run |
| Home/About identity | EffectStrike strings and existing PWDS logo integrated in desktop source | Static review only; visual test pending |

## Checks completed on this branch

- Exact upstream commit and repository source structure verified.
- Existing PWDS PNG verified by Git blob hash against the public source.
- No local compilation or large downloads. The complete shallow checkout used under 30 MB of working-tree files.
- Upstream auto-running GitHub Actions workflows removed; no hosted runner has been started or charged.
- Static source and licence checks are recorded in the draft PR. The current branch is **not** a tested build or a release.

## Open work before an installer or production claim

1. Obtain the suite build slot, then run a bounded `cargo check`/test pass and one representative create → import → animate → save/reopen → render path, inspecting output independently.
2. Review Home/About at multiple sizes, and check keyboard, dialogs, project recovery, and failure paths on Windows. No native interaction has been tested in this fork.
3. Replace the current upstream app icon with an approved illustrated creative-category EffectStrike asset when one is supplied. Review other EffectCraft artwork, localized text, web frontend, CLI naming, and installer metadata. Packaging source is inherited and is not approved for distribution yet.
4. Decide a cost-free hosted test route and budget before adding any automatic CI workflows. No paid runners or services are authorized.
5. Upstream is alpha and reports substantial professional workflow gaps. `.aep`/`.aepx` and After Effects plug-ins are unsupported; this fork does not claim parity or production readiness.
