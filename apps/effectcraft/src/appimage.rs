//! Desktop integration for the AppImage. Wayland compositors (GNOME, KDE…) show a window's icon
//! from the desktop entry named by its app ID (`ai.storyteller.effectcraft.desktop`). The .deb,
//! .rpm and Flatpak packages install one; an AppImage installs nothing, so its window got the
//! generic icon. Started from an AppImage (`$APPIMAGE`), the app adds a hidden entry
//! (`NoDisplay=true`: no menu item) pointing at it and its icon under the user's data folder
//! (`$XDG_DATA_HOME`, else `~/.local/share`). Nothing is written when another entry already
//! provides the ID (an installed package, or the user's own), and an entry it wrote is kept up to
//! date when the AppImage moves. Failures are only logged: the icon is cosmetic.

use std::io::Read;
use std::path::{Path, PathBuf};

pub const APP_ID: &str = "ai.storyteller.effectcraft";

/// Marks entries this module wrote (it never touches others).
const MARKER: &str = "X-EffectCraft-AppImage=true";

/// A path as one `Exec` argument (desktop entry spec: quoted, with `"`, `` ` ``, `$` and `\`
/// escaped; `%` doubled so it isn't a field code).
pub fn exec_arg(path: &str) -> String {
    let mut s = String::with_capacity(path.len() + 2);
    s.push('"');
    for c in path.chars() {
        // The quoting escape is a backslash, which the key file's string syntax escapes again:
        // `"` is written `\\"`, a backslash `\\\\`.
        match c {
            '"' | '`' | '$' => {
                s.push_str("\\\\");
                s.push(c);
            }
            '\\' => s.push_str("\\\\\\\\"),
            '%' => s.push_str("%%"),
            '\n' | '\r' => {}
            c => s.push(c),
        }
    }
    s.push('"');
    s
}

/// The hidden desktop entry for the AppImage at `appimage`.
pub fn entry(appimage: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=EffectStrike\nComment=Motion graphics and visual effects compositor\nExec={} %F\nIcon={APP_ID}\nTerminal=false\nStartupWMClass={APP_ID}\nNoDisplay=true\n{MARKER}\n",
        exec_arg(appimage)
    )
}

/// Write the entry and `icon` (a 256 px PNG) under `data_home`, unless one of `data_dirs` (or
/// `data_home` itself, with an entry we didn't write) already has the ID's entry. Returns whether
/// anything was written.
pub fn integrate(appimage: &str, data_home: &Path, data_dirs: &[PathBuf], icon: &[u8]) -> std::io::Result<bool> {
    let name = format!("{APP_ID}.desktop");
    if data_dirs.iter().any(|d| d.join("applications").join(&name).exists()) {
        return Ok(false);
    }
    let path = data_home.join("applications").join(&name);
    let text = entry(appimage);
    let icon_path = data_home.join("icons/hicolor/256x256/apps").join(format!("{APP_ID}.png"));
    require_regular_or_missing(&path)?;
    require_regular_or_missing(&icon_path)?;
    match std::fs::read_to_string(&path) {
        Ok(old) if !owns_entry(&old) => return Ok(false),
        Ok(old) if old == text && icon_matches(&icon_path, icon) => return Ok(false),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    write_atomic(&icon_path, icon)?;
    write_atomic(&path, text.as_bytes())?;
    Ok(true)
}

fn require_regular_or_missing(path: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Ok(_) => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "desktop integration path is not a regular file")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Recognize the generated entry, not a marker mentioned in a comment or another section.
fn owns_entry(text: &str) -> bool {
    let mut sections = text.lines().filter(|line| line.starts_with('['));
    if sections.next() != Some("[Desktop Entry]") || sections.next().is_some() {
        return false;
    }
    [
        ("X-EffectCraft-AppImage=", MARKER),
        ("Type=", "Type=Application"),
        ("Icon=", "Icon=ai.storyteller.effectcraft"),
        ("StartupWMClass=", "StartupWMClass=ai.storyteller.effectcraft"),
        ("NoDisplay=", "NoDisplay=true"),
    ]
    .iter()
    .all(|(key, value)| {
        let mut values = text.lines().filter(|line| line.starts_with(key));
        values.next() == Some(*value) && values.next().is_none()
    })
}

/// Compare only the expected bytes plus one; an existing file cannot force an unbounded read.
fn icon_matches(path: &Path, expected: &[u8]) -> bool {
    let Ok(file) = std::fs::File::open(path) else { return false };
    let limit = u64::try_from(expected.len()).unwrap_or(u64::MAX).saturating_add(1);
    let mut actual = Vec::new();
    file.take(limit).read_to_end(&mut actual).is_ok() && actual == expected
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("part");
    // Never follow or truncate an existing temporary file (which may belong to the user).
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp)?;
    let result = std::io::Write::write_all(&mut file, bytes);
    drop(file);
    if let Err(error) = result {
        let _ = std::fs::remove_file(&tmp);
        return Err(error);
    }
    if let Err(error) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(error);
    }
    Ok(())
}

/// [`integrate`] for this process: when started from an AppImage, with the XDG data folders of
/// the environment.
pub fn integrate_from_env(icon: &[u8]) {
    let Some(appimage) = std::env::var_os("APPIMAGE").map(|a| a.to_string_lossy().into_owned()) else { return };
    let icon = match desktop_icon(icon) {
        Ok(icon) => icon,
        Err(e) => {
            log::warn!("AppImage: could not prepare the product icon: {e}");
            return;
        }
    };
    let env_path = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    let Some(data_home) = env_path("XDG_DATA_HOME").or_else(|| env_path("HOME").map(|h| h.join(".local/share"))) else { return };
    let dirs = std::env::var("XDG_DATA_DIRS").ok().filter(|v| !v.is_empty()).unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    let data_dirs: Vec<PathBuf> = dirs.split(':').filter(|d| !d.is_empty()).map(PathBuf::from).collect();
    match integrate(&appimage, &data_home, &data_dirs, &icon) {
        Ok(true) => log::info!("AppImage: desktop entry and icon written to {}", data_home.display()),
        Ok(false) => {}
        Err(e) => log::warn!("AppImage: could not write the desktop entry: {e}"),
    }
}

/// Export the approved runtime artwork at the size promised by the hicolor directory.
fn desktop_icon(source: &[u8]) -> Result<Vec<u8>, image::ImageError> {
    let image = image::load_from_memory(source)?.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
    let mut output = std::io::Cursor::new(Vec::new());
    image.write_to(&mut output, image::ImageFormat::Png)?;
    Ok(output.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn symlinked_icon_entry_or_temporary_file_preserves_target() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!("ec-appimage-symlink-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let entry_path = root.join("applications/ai.storyteller.effectcraft.desktop");
        let icon_path = root.join("icons/hicolor/256x256/apps/ai.storyteller.effectcraft.png");
        let target = root.join("user-sentinel");
        assert!(integrate("/app", &root, &[], b"old").unwrap());
        std::fs::write(&target, b"keep").unwrap();
        for path in [&icon_path, &entry_path] {
            let original = std::fs::read(path).unwrap();
            std::fs::remove_file(path).unwrap();
            symlink(&target, path).unwrap();
            assert!(integrate("/app", &root, &[], b"new").is_err());
            assert_eq!(std::fs::read(&target).unwrap(), b"keep");
            std::fs::remove_file(path).unwrap();
            std::fs::write(path, original).unwrap();
        }
        symlink(&target, icon_path.with_extension("part")).unwrap();
        assert!(integrate("/app", &root, &[], b"new").is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"keep");
        assert_eq!(std::fs::read(&icon_path).unwrap(), b"old");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn ambiguous_or_unreadable_entries_never_replace_icons() {
        let root = std::env::temp_dir().join(format!("ec-appimage-ownership-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let path = root.join("applications/ai.storyteller.effectcraft.desktop");
        let icon = root.join("icons/hicolor/256x256/apps/ai.storyteller.effectcraft.png");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(icon.parent().unwrap()).unwrap();
        std::fs::write(&icon, b"user-icon").unwrap();
        for text in [
            format!("[Desktop Entry]\nName=User\n# {MARKER}\n"),
            format!("{}Icon=another-app\n", entry("/old/app")),
            format!("{}[Another Group]\n", entry("/old/app")),
        ] {
            std::fs::write(&path, &text).unwrap();
            assert!(!integrate("/new/app", &root, &[], b"new-icon").unwrap());
            assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
            assert_eq!(std::fs::read(&icon).unwrap(), b"user-icon");
        }
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(integrate("/new/app", &root, &[], b"new-icon").is_err());
        assert!(path.is_dir());
        assert_eq!(std::fs::read(&icon).unwrap(), b"user-icon");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn product_artwork_exports_a_valid_transparent_256px_icon() {
        let source = include_bytes!("../../../assets/branding/EffectStrike-logo.png");
        let png = desktop_icon(source).unwrap();
        let decoded = image::load_from_memory(&png).unwrap().into_rgba8();
        assert_eq!(decoded.dimensions(), (256, 256));
        let expected = image::load_from_memory(source).unwrap().resize_exact(256, 256, image::imageops::FilterType::Lanczos3).into_rgba8();
        assert_eq!(decoded, expected);
        assert_eq!(decoded.get_pixel(0, 0).0[3], 0);
        assert!(decoded.pixels().any(|pixel| pixel.0[3] > 0));
        assert!(desktop_icon(b"not a PNG").is_err());
    }

    #[test]
    fn same_path_refreshes_only_our_outdated_icon() {
        let root = std::env::temp_dir().join(format!("ec-appimage-icon-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let icon_path = root.join("icons/hicolor/256x256/apps/ai.storyteller.effectcraft.png");
        let entry_path = root.join("applications/ai.storyteller.effectcraft.desktop");
        assert!(integrate("/app/EffectStrike.AppImage", &root, &[], b"old-icon").unwrap());
        let original_entry = std::fs::read(&entry_path).unwrap();
        assert!(integrate("/app/EffectStrike.AppImage", &root, &[], b"product-icon").unwrap());
        assert_eq!(std::fs::read(&icon_path).unwrap(), b"product-icon");
        assert_eq!(std::fs::read(&entry_path).unwrap(), original_entry);
        assert!(!integrate("/app/EffectStrike.AppImage", &root, &[], b"product-icon").unwrap());
        std::fs::write(&icon_path, b"product-icon-with-extra-bytes").unwrap();
        assert!(!icon_matches(&icon_path, b"product-icon"));
        assert!(integrate("/app/EffectStrike.AppImage", &root, &[], b"product-icon").unwrap());
        std::fs::write(&icon_path, b"product").unwrap();
        assert!(!icon_matches(&icon_path, b"product-icon"));
        assert!(integrate("/app/EffectStrike.AppImage", &root, &[], b"product-icon").unwrap());
        std::fs::write(&entry_path, "[Desktop Entry]\nName=User choice\n").unwrap();
        assert!(!integrate("/app/EffectStrike.AppImage", &root, &[], b"replacement").unwrap());
        assert_eq!(std::fs::read(&icon_path).unwrap(), b"product-icon");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn exec_arguments_are_quoted_and_escaped() {
        assert_eq!(exec_arg("/home/a/EffectCraft.AppImage"), "\"/home/a/EffectCraft.AppImage\"");
        assert_eq!(exec_arg("/a b/$x\"y`z\\w%"), "\"/a b/\\\\$x\\\\\"y\\\\`z\\\\\\\\w%%\"");
        let e = entry("/opt/EffectCraft.AppImage");
        assert!(e.contains("Name=EffectStrike\n"));
        assert!(e.contains("Exec=\"/opt/EffectCraft.AppImage\" %F\n") && e.contains("NoDisplay=true\n") && e.contains("Icon=ai.storyteller.effectcraft\n"));
    }

    #[test]
    fn writes_once_follows_moves_and_leaves_other_entries_alone() {
        let root = std::env::temp_dir().join(format!("ec-appimage-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (home, system) = (root.join("home"), root.join("system"));
        let entry_path = home.join("applications/ai.storyteller.effectcraft.desktop");
        let icon_path = home.join("icons/hicolor/256x256/apps/ai.storyteller.effectcraft.png");
        // First start: entry and icon written; again: nothing to do.
        assert!(integrate("/a/EffectCraft.AppImage", &home, std::slice::from_ref(&system), b"png").unwrap());
        assert_eq!(std::fs::read(&icon_path).unwrap(), b"png");
        assert!(std::fs::read_to_string(&entry_path).unwrap().contains("Exec=\"/a/EffectCraft.AppImage\""));
        assert!(!integrate("/a/EffectCraft.AppImage", &home, std::slice::from_ref(&system), b"png").unwrap());
        // The AppImage moved: the entry follows it.
        assert!(integrate("/b/EffectCraft.AppImage", &home, std::slice::from_ref(&system), b"png").unwrap());
        assert!(std::fs::read_to_string(&entry_path).unwrap().contains("Exec=\"/b/EffectCraft.AppImage\""));
        // The user's own entry is never replaced.
        std::fs::write(&entry_path, "[Desktop Entry]\nName=Mine\n").unwrap();
        assert!(!integrate("/c/EffectCraft.AppImage", &home, std::slice::from_ref(&system), b"png").unwrap());
        assert_eq!(std::fs::read_to_string(&entry_path).unwrap(), "[Desktop Entry]\nName=Mine\n");
        // An installed package provides the entry: nothing is written.
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(system.join("applications")).unwrap();
        std::fs::write(system.join("applications/ai.storyteller.effectcraft.desktop"), "x").unwrap();
        assert!(!integrate("/a/EffectCraft.AppImage", &home, std::slice::from_ref(&system), b"png").unwrap());
        assert!(!entry_path.exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
