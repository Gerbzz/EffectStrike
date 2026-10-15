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
| Create and edit a composition with layers | Present: project model, engine commands, egui timeline/viewer | Headless CLI created a 160x90 comp with a solid and imported PNG; desktop editing not tested. |
| Import footage and save/reopen `.ecproj` | Present: media, project serialization, UI and CLI commands | Generated PNG imported; `.ecproj` saved, reopened, and rendered on Ubuntu. Video/RAW import not tested. |
| Animate with keyframes and graph editor | Present: keyframe crate and UI panels | Two imported-layer position keys saved/reopened; frames at 0 and 0.5 seconds differed. Graph editor not tested. |
| Apply effects and expressions | Present: effects registry, expression engine, controls | Not run |
| Use cameras, lights, and 3D layers | Present: model/render crates and viewer controls | Not run |
| Queue and export a render | Present: export/render queue and CLI | CPU `render-frame` made a 160x90 PNG; render queue exported three PNG frames. Video formats not tested. |
| Home/About identity | EffectStrike strings and existing PWDS logo integrated in desktop source | Home captured at 1280x800 on Ubuntu 24.04/Xvfb/Mesa llvmpipe: name, PWDS mark, and primary actions visible without obvious clipping. About is source-reviewed, not visually tested. |

## Checks completed on this branch

- Exact upstream commit and repository source structure verified.
- Existing PWDS PNG verified by Git blob hash against the public source.
- No local compilation or large downloads. The complete shallow checkout used under 30 MB of working-tree files.
- Upstream release workflows removed. The fork's separate [bounded public Ubuntu 24.04 workflow](../.github/workflows/effectstrike-smoke.yml) uses Rust 1.95.0, read-only repository permissions, a 55-minute standard runner, and one-day retention for small screenshots. [GitHub says standard hosted runners are free for public repositories](https://docs.github.com/en/billing/concepts/product-billing/github-actions); no paid runner, cache, or service was used.
- [Run 38713229832](https://github.com/Gerbzz/EffectStrike/actions/runs/38713229832) passed full desktop/CLI builds, synthetic import/save/reopen/PNG render, and 25 CLI tests. Its 160x90 renderer frame was downloaded and visually inspected.
- [Run 38715306746](https://github.com/Gerbzz/EffectStrike/actions/runs/38715306746) passed the full builds, product identity, two-keyframe animation, three-frame PNG queue export, 25 CLI tests, and two Linux desktop-entry tests. The Home capture then failed because the runner lacked `libxkbcommon-x11.so.0`; commit `11ffad2` added and verified that runtime library.
- [Run 38717418028](https://github.com/Gerbzz/EffectStrike/actions/runs/38717418028) passed every step including native Home startup and screenshot capture. Its downloaded Home image was visually inspected at 1280x800; the app used an Xvfb display and Mesa llvmpipe software OpenGL. The renderer PNG was also visually inspected. These are Linux hosted results, not Windows or macOS results.
- The exact [Home capture](images/effectstrike-home-ubuntu24.png) and [CPU renderer frame](images/effectstrike-synthetic-render.png) from that run are preserved with attribution sidecars and source/environment details. The documentation-only image addition follows the tested code commit; it does not change application behavior.
- Static source and licence checks are recorded in the draft PR. This is a **tested synthetic baseline, not a release**.

## Open work before an installer or production claim

1. Use the suite build slot when available for native Windows verification. Expand the hosted synthetic path to representative video/footage, effects, text, graph-editor interaction, save/reopen, and video export; inspect outputs independently.
2. Review Home/About at multiple sizes, and check keyboard, dialogs, project recovery, and failure paths on Windows. The Home screen has a Linux startup screenshot, but no interactive desktop workflow or About screenshot has been tested.
3. Replace the current upstream app icon with an approved illustrated creative-category EffectStrike asset when one is supplied. Review other EffectCraft artwork, localized text, web frontend, CLI naming, and installer metadata. Packaging source is inherited and is not approved for distribution yet.
4. Keep hosted validation on standard public runners with bounded jobs and short screenshot retention. Do not add paid runners, services, or long-lived build caches.
5. Upstream is alpha and reports substantial professional workflow gaps. `.aep`/`.aepx` and After Effects plug-ins are unsupported; this fork does not claim parity or production readiness.
