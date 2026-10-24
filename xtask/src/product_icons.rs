//! Reproducible, geometry-preserving exports of the accepted product PNG.
//! ICNS is assembled separately with macOS iconutil from the emitted iconset.

use std::collections::BTreeMap;
use std::path::Path;

const MASTER: &str = "assets/branding/EffectStrike-logo.png";
const VECTOR: &str = "assets/branding/EffectStrike-logo.svg";
const OUTPUT: &str = "assets/branding/icons";
const APP_ID: &str = "ai.storyteller.effectcraft";
const HICOLOR: &[u32] = &[16, 24, 32, 48, 64, 128, 256, 512];
const ICO: &[u32] = &[16, 20, 24, 32, 40, 48, 64, 128, 256];

fn png(image: &image::DynamicImage, size: u32) -> Result<Vec<u8>, String> {
    let resized = image.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
    let mut bytes = std::io::Cursor::new(Vec::new());
    resized.write_to(&mut bytes, image::ImageFormat::Png).map_err(|error| error.to_string())?;
    Ok(bytes.into_inner())
}

fn exports(master: &[u8], vector: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    // Check dimensions before image decoding; only the fixed-size approved source is accepted.
    if super::ico::png_size(master)? != (1024, 1024) {
        return Err("product master must be a 1024 by 1024 PNG".into());
    }
    let image = image::load_from_memory_with_format(master, image::ImageFormat::Png).map_err(|error| error.to_string())?;
    if image.color() != image::ColorType::Rgba8 {
        return Err("product master must retain RGBA transparency".into());
    }
    let mut sizes = BTreeMap::new();
    for size in [16, 20, 24, 32, 40, 48, 64, 128, 256, 512] {
        sizes.insert(size, png(&image, size)?);
    }
    // Preserve the original bytes at native resolution, rather than re-encoding them.
    sizes.insert(1024, master.to_vec());
    let mut out = BTreeMap::new();
    for size in HICOLOR {
        out.insert(format!("hicolor/{size}x{size}/apps/{APP_ID}.png"), sizes.get(size).ok_or("missing PNG size")?.clone());
    }
    out.insert(format!("hicolor/scalable/apps/{APP_ID}.svg"), vector.to_vec());
    let ico_pngs = ICO.iter().map(|size| sizes.get(size).cloned().ok_or("missing ICO size")).collect::<Result<Vec<_>, _>>()?;
    out.insert("EffectStrike.ico".into(), super::ico::pack(&ico_pngs)?);
    for size in [16, 32, 128, 256, 512] {
        out.insert(format!("EffectStrike.iconset/icon_{size}x{size}.png"), sizes.get(&size).ok_or("missing iconset size")?.clone());
        out.insert(format!("EffectStrike.iconset/icon_{size}x{size}@2x.png"), sizes.get(&(size * 2)).ok_or("missing double-resolution size")?.clone());
    }
    Ok(out)
}

pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    if let [flag, directory] = args
        && flag == "--verify-iconset"
    {
        return verify_iconset(root, Path::new(directory));
    }
    let write = match args {
        [] => false,
        [flag] if flag == "--check" => false,
        [flag] if flag == "--write" => true,
        _ => return Err("usage: cargo xtask product-icons [--check|--write|--verify-iconset DIRECTORY]".into()),
    };
    let master = std::fs::read(root.join(MASTER)).map_err(|error| format!("{MASTER}: {error}"))?;
    let vector = std::fs::read(root.join(VECTOR)).map_err(|error| format!("{VECTOR}: {error}"))?;
    let generated = exports(&master, &vector)?;
    for (relative, bytes) in &generated {
        let path = root.join(OUTPUT).join(relative);
        if write {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            std::fs::write(&path, bytes).map_err(|error| format!("{}: {error}", path.display()))?;
        } else if std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))? != *bytes {
            return Err(format!("{} differs from the accepted-master export", path.display()));
        }
    }
    println!(
        "product-icons: {} deterministic files {}; native ICNS is checked separately with iconutil",
        generated.len(),
        if write { "written" } else { "verified" }
    );
    Ok(())
}

fn verify_iconset(root: &Path, directory: &Path) -> Result<(), String> {
    let master = std::fs::read(root.join(MASTER)).map_err(|error| error.to_string())?;
    let generated = exports(&master, &[])?;
    for (relative, expected) in generated.iter().filter(|(name, _)| name.starts_with("EffectStrike.iconset/")) {
        let name = relative.strip_prefix("EffectStrike.iconset/").ok_or("invalid iconset name")?;
        let path = directory.join(name);
        let actual = std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        if super::ico::png_size(&actual)? != super::ico::png_size(expected)? {
            return Err(format!("{name}: wrong iconset dimensions"));
        }
        let actual = image::load_from_memory_with_format(&actual, image::ImageFormat::Png).map_err(|error| error.to_string())?.into_rgba8();
        let expected = image::load_from_memory_with_format(expected, image::ImageFormat::Png).map_err(|error| error.to_string())?.into_rgba8();
        if actual != expected {
            return Err(format!("{name}: iconutil roundtrip changed approved pixels"));
        }
    }
    println!("product-icons: all ten iconutil roundtrip PNGs match the approved pixels");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn master() -> &'static [u8] {
        include_bytes!("../../assets/branding/EffectStrike-logo.png")
    }

    #[test]
    fn every_export_preserves_the_master_pixels_at_its_declared_size() {
        let original = image::load_from_memory(master()).unwrap();
        let generated = exports(master(), b"unchanged-vector").unwrap();
        assert_eq!(generated.len(), 20);
        for (path, bytes) in generated.iter().filter(|(path, _)| path.ends_with(".png")) {
            let decoded = image::load_from_memory(bytes).unwrap().into_rgba8();
            let size = decoded.width();
            assert_eq!(decoded.height(), size, "{path}");
            assert_eq!(decoded, original.resize_exact(size, size, image::imageops::FilterType::Lanczos3).into_rgba8(), "{path}");
            assert_eq!(decoded.get_pixel(0, 0).0[3], 0, "{path}");
            assert!(decoded.pixels().any(|pixel| pixel.0[3] != 0), "{path}");
        }
        assert_eq!(generated["EffectStrike.iconset/icon_512x512@2x.png"], master());
        assert_eq!(generated[&format!("hicolor/scalable/apps/{APP_ID}.svg")], b"unchanged-vector");
    }

    #[test]
    fn ico_contains_all_expected_png_payloads() {
        let generated = exports(master(), b"vector").unwrap();
        let ico = &generated["EffectStrike.ico"];
        assert_eq!(&ico[..6], &[0, 0, 1, 0, 9, 0]);
        let original = image::load_from_memory(master()).unwrap();
        for (index, size) in ICO.iter().enumerate() {
            let entry = 6 + index * 16;
            let length = u32::from_le_bytes(ico[entry + 8..entry + 12].try_into().unwrap()) as usize;
            let offset = u32::from_le_bytes(ico[entry + 12..entry + 16].try_into().unwrap()) as usize;
            let decoded = image::load_from_memory(&ico[offset..offset + length]).unwrap().into_rgba8();
            assert_eq!(decoded.dimensions(), (*size, *size));
            assert_eq!(decoded, original.resize_exact(*size, *size, image::imageops::FilterType::Lanczos3).into_rgba8());
        }
    }

    #[test]
    fn invalid_or_wrong_size_master_is_rejected() {
        assert!(exports(b"not PNG", b"svg").is_err());
        let small = png(&image::load_from_memory(master()).unwrap(), 16).unwrap();
        assert!(exports(&small, b"svg").is_err());
        let mut corrupt = master().to_vec();
        corrupt.truncate(30);
        assert!(exports(&corrupt, b"svg").is_err());
    }

    #[test]
    fn iconset_verification_rejects_missing_or_changed_output() {
        let root = crate::root();
        let directory = std::env::temp_dir().join(format!("effectstrike-iconset-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        assert!(verify_iconset(&root, &directory).is_err());
        for (name, bytes) in exports(master(), b"").unwrap().iter().filter(|(name, _)| name.starts_with("EffectStrike.iconset/")) {
            std::fs::write(directory.join(name.strip_prefix("EffectStrike.iconset/").unwrap()), bytes).unwrap();
        }
        assert!(verify_iconset(&root, &directory).is_ok());
        std::fs::write(directory.join("icon_16x16.png"), b"broken").unwrap();
        assert!(verify_iconset(&root, &directory).is_err());
        let _ = std::fs::remove_dir_all(directory);
    }
}
