# EffectStrike

EffectStrike is an independent PWDS motion graphics and visual effects application built on the **complete [EffectCraft](https://github.com/storytold/effectcraft) Rust compositor**. This repository contains the native desktop app, project and render engine, egui interface, CLI, web frontend source, and tests. It does not reduce EffectCraft to a few crates or a mock interface.

The foundation is pinned to upstream commit [`0311bdc`](https://github.com/storytold/effectcraft/commit/0311bdc34d0dc10c2d18ff7875b4f52d483e5e27) (15 October 2026). The source snapshot is published as a new root commit so that upstream's separately licensed ArtCraft trademark files are absent from this repository's history. See [NOTICE](NOTICE), [ATTRIBUTION.md](ATTRIBUTION.md), [LICENSE-MIT](LICENSE-MIT), and [LICENSE-APACHE](LICENSE-APACHE). EffectStrike is not made or endorsed by the ArtCraft Team.

## Workflows present in the foundation

The upstream source implements composition and layer editing, footage import, keyframes and graph editing, effects, expressions, 3D cameras and lights, and a render queue. Its CLI can render and edit projects headlessly. These are **upstream source capabilities**, not a claim that this fork has passed acceptance testing. The source retains the `.ecproj` project format and internal `effectcraft-*` Rust crate names for compatibility.

The desktop build currently uses the existing `effectcraft` Cargo package and binary name:

```sh
cargo run --release -p effectcraft
cargo run --release -p effectcraft -- --demo
cargo run --release -p effectcraft-cli -- render --out demo.mp4
```

Rust 1.95 or newer is required. The release-mode `cargo run` commands above are documented from upstream and have not been run for this fork. A hosted Ubuntu 24.04 job has built both debug binaries and exercised the CLI with a synthetic project. No installer or signed release is offered here yet.

## Integration status

- The desktop window, Home screen, About dialog, Help links, and Windows resource description display EffectStrike and point feedback to this repository.
- The approved PWDS company mark appears on a light plaque on Home and About. A distinct illustrated EffectStrike app icon has not been supplied; the openly licensed upstream app icon remains in the source for now. See [brand attribution](assets/branding/PWDS-logo.png.attribution).
- ArtCraft trademark image files were removed. The upstream code, legal notices, third-party attributions, and open licences remain.
- Upstream GitHub Actions workflows were removed so publishing this source does not start unapproved hosted builds or release jobs.
- No local compile, render, native interaction, or installer test has run; another suite app owns the local build slot. [Bounded standard-runner validation](https://github.com/Gerbzz/EffectStrike/actions/runs/38717418028) built the desktop and CLI apps, imported a generated PNG, saved/reopened a composition, animated it with two keys, rendered a frame and a three-frame PNG sequence, passed 25 CLI and two Linux desktop-entry tests, and captured the native Home screen on an Ubuntu/Xvfb software display. The [verification ledger](docs/EFFECTSTRIKE_STATUS.md) records the environment and limits.

EffectCraft describes itself as an alpha compositor and cautions against relying on it for client work. EffectStrike is also pre-release. Existing After Effects `.aep` and `.aepx` projects and plug-ins are not supported by the upstream foundation. Internal menus, translations, web branding, app icon, and packaging still need fork-specific review before distribution.

## Next checkpoints

1. Extend the passing synthetic project path to representative footage, effects, text, graph editing, and video export. Validate native Windows operation when the suite build slot is available.
2. Inspect Home and About at more window sizes and on Windows/macOS; test dialogs, keyboard actions, and accessibility. The hosted Linux Home capture is a visual baseline, not an interactive acceptance test.
3. Complete application identity across CLI, web, localized strings, icons, packaging, and installer metadata using approved creative-category artwork.
4. Compare output against simple independent reference cases before claiming reliable production workflows.

The remaining ArtCraft counterparts and foundation choices are recorded in [the suite coverage plan](docs/STRIKE_COVERAGE_PLAN.md).

## Screenshots from the verified hosted run

The [green Ubuntu 24.04 run](https://github.com/Gerbzz/EffectStrike/actions/runs/38717418028) captured these exact images at code revision `11ffad2`. The Home image used Xvfb at 1280x800 with Mesa llvmpipe software OpenGL; the render image came from the CPU CLI compositor using generated media. They do not demonstrate Windows/macOS behavior or an interactive editing session. See each image's attribution sidecar and [asset index](ATTRIBUTION.md).

![EffectStrike Home on hosted Ubuntu 24.04, with the PWDS company mark](docs/images/effectstrike-home-ubuntu24.png)

![Synthetic 160x90 composition frame produced by EffectStrike's CPU renderer](docs/images/effectstrike-synthetic-render.png)
