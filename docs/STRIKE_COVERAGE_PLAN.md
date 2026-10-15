# Remaining Craft app counterparts

Checked against the [official 13-app Crafting Apps lineup](https://getartcraft.com/apps) on 15 October 2026 and the public `Gerbzz` repository list. A matching repository name is only a catalog check, not proof of feature parity or release quality.

| Official app | Strike counterpart | Public catalog state |
| --- | --- | --- |
| PhotoCraft | PhotoStrike | Exists |
| VectorCraft | VectorStrike | Exists |
| FilmCraft | ClipStrike | Exists; workflow comparison still needed |
| LightCraft | **LightStrike** | [Full-source public foundation](https://github.com/Gerbzz/LightStrike); [draft verification PR](https://github.com/Gerbzz/LightStrike/pull/1) |
| PdfCraft | PDFStrike | Exists |
| EffectCraft | **EffectStrike** | Full-source integration in this repository; [synthetic Linux build and workflow passed](https://github.com/Gerbzz/EffectStrike/actions/runs/38717418028), with release gaps documented |
| DesignCraft | **DesignStrike** | Missing |
| FigCraft | **FigStrike** | Missing |
| WordCraft | WordStrike | Exists |
| SoundCraft | AudioStrike | Exists |
| DeckCraft | SlideStrike | Exists |
| CADCraft | CADStrike | Exists |
| GridCraft | GridStrike | Exists |

## Order and acceptance paths

1. **LightStrike:** inspect the complete [LightCraft photo library and raw developer](https://getartcraft.com/apps/lightcraft) and [source](https://github.com/storytold/lightcraft). It already has a Rust app foundation, so evaluate its actual import, non-destructive edit, catalog, and export behavior before considering another engine. Preserve originals. Gate: import a supported raw and a JPEG, make reversible light/color/mask adjustments, save/reopen the catalog state, export, and compare output and original bytes. Check the library UI with a meaningful catalog, not a blank mock.
2. **DesignStrike:** inspect the complete [DesignCraft page-layout app](https://getartcraft.com/apps/designcraft) and [source](https://github.com/storytold/designcraft). Gate: create a multi-page publication with linked text, styles, images and parent-page elements; save/reopen; export PDF; inspect all pages and overset behavior. Check typography and print controls in the native UI.
3. **FigStrike:** inspect the complete [FigCraft interface-design app](https://getartcraft.com/apps/figcraft) and [source](https://github.com/storytold/figcraft). Gate: build responsive frames with auto layout, components/variants, styles or variables and a prototype; save/reopen; export assets; verify prototype interactions and the design panel visually.

For each app: first search current local work and Gerbzz repos, pin the upstream revision, read the actual source and licences, remove restricted upstream marks from the **published history**, reuse approved creative-category artwork without generating replacement logos casually, and maintain notices and asset attribution. Keep internal formats compatible until a migration is designed. Create a public repository only after it holds the full application source and an honest status page. Use an isolated draft branch for identity and workflow work, and start no hosted jobs until a free bounded path is verified. No local builds while another suite task owns the build slot or if storage would cross the 20 GiB free-space floor.

## Upstream source audit (15 October 2026)

All three official pages point to full Rust workspaces with native egui, CLI, and web app paths. Each source declares `MIT OR Apache-2.0`; preserve its NOTICE and asset attribution. Their separate ArtCraft brand licences require modified forks to remove ArtCraft marks. The illustrated app icons have open licences, but their attribution still needs to travel with them. LightStrike now holds the full LightCraft application snapshot; DesignStrike and FigStrike have not been created from this audit.

| Candidate | Inspected source pin | Important source-reported gap |
| --- | --- | --- |
| LightCraft | [`0da998f24dfc03069d49867f8877af2bf7c51f94`](https://github.com/storytold/lightcraft/tree/0da998f24dfc03069d49867f8877af2bf7c51f94) | Alpha; [camera color, compressed RAW variants, and lens profiles](https://github.com/storytold/lightcraft/blob/0da998f24dfc03069d49867f8877af2bf7c51f94/docs/gaps.md) need concrete test images. |
| DesignCraft | [`a930276fd7462f3d9abd2fbb38544c94c0c911c8`](https://github.com/storytold/designcraft/tree/a930276fd7462f3d9abd2fbb38544c94c0c911c8) | Pre-alpha; the [Primary Text Frame page blocker](https://github.com/storytold/designcraft/blob/a930276fd7462f3d9abd2fbb38544c94c0c911c8/docs/gaps.md) needs a fix before a publication workflow can pass. Its [asset manifest](https://github.com/storytold/designcraft/blob/a930276fd7462f3d9abd2fbb38544c94c0c911c8/ASSETS.md) also identifies a restricted ArtCraft mark drawn in UI code; deleting brand files alone is insufficient. |
| FigCraft | [`8482c3e85bc5f2a430e2f3be2826674872f6d084`](https://github.com/storytold/figcraft/tree/8482c3e85bc5f2a430e2f3be2826674872f6d084) | Alpha; [`.fig` import and live sharing](https://github.com/storytold/figcraft/blob/8482c3e85bc5f2a430e2f3be2826674872f6d084/ROADMAP.md) are preliminary. Review any public-sharing default before distribution. |

An existing cel-animation app such as AnimateStrike does not cover EffectCraft's compositing, VFX, 3D, effects, and render-queue workflow.
