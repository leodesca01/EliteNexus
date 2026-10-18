//! Where the game is installed, and whether a 3Dmigoto shader mod such as EDHM sits next to it.
//!
//! EDHM (Elite Dangerous HUD Mod) recolours the HUD with its own shaders and needs the game's
//! colour matrix left at the standard values; its installer resets it. So the HUD colours page
//! has to know when it's there.

use std::path::{Path, PathBuf};

/// One game client folder: the one holding `EliteDangerous64.exe` (`Products\elite-dangerous-odyssey-64`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInstall {
    pub folder: PathBuf,
    /// "Steam", "Epic" or "Frontier".
    pub store: &'static str,
    pub shader_mod: Option<ShaderMod>,
}

/// A 3Dmigoto mod found in a game folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShaderMod {
    /// EDHM's own settings folder is there (otherwise some other 3Dmigoto mod).
    pub edhm: bool,
    /// `d3d11.dll` is in place; EDHM-UI's on/off switch renames it to `d3d11.dll.disabled`.
    pub enabled: bool,
}

impl GameInstall {
    pub fn is_live(&self) -> bool {
        self.folder.file_name().is_some_and(|n| n.to_string_lossy().contains("odyssey"))
    }
}

/// What a game folder holds besides the game: `None` when no 3Dmigoto mod is installed.
pub fn shader_mod(folder: &Path) -> Option<ShaderMod> {
    folder.join("d3dx.ini").is_file().then(|| ShaderMod { edhm: folder.join("EDHM-ini").is_dir(), enabled: folder.join("d3d11.dll").is_file() })
}

/// Every game client folder found in the usual Steam, Epic and Frontier locations. On Linux and macOS: every Steam
/// library (Proton, NotProton), and Program Files in the game's Wine prefix (a Frontier or Epic launcher under
/// Wine or CrossOver).
pub fn find_installs() -> Vec<GameInstall> {
    let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
    let mut roots: Vec<(PathBuf, &'static str)> = nexus_journal::steam_libraries().into_iter().map(|lib| (lib.join("steamapps").join("common").join("Elite Dangerous"), "Steam")).collect();
    let program_files: Vec<PathBuf> = match nexus_journal::paths::wine_drive_c() {
        Some(c) => vec![c.join("Program Files (x86)"), c.join("Program Files")],
        None => [env("ProgramFiles(x86)"), env("ProgramFiles")].into_iter().flatten().collect(),
    };
    for pf in program_files {
        roots.push((pf.join("Frontier").join("EDLaunch"), "Frontier"));
        roots.push((pf.join("Frontier").join("Elite Dangerous Launcher"), "Frontier"));
        roots.push((pf.join("Epic Games").join("EliteDangerous"), "Epic"));
    }
    if let Some(data) = env("ProgramData") {
        let manifests = data.join("Epic").join("EpicGamesLauncher").join("Data").join("Manifests");
        for e in std::fs::read_dir(manifests).into_iter().flatten().flatten() {
            if let Some(loc) = std::fs::read_to_string(e.path()).ok().and_then(|t| epic_location(&t)) {
                roots.push((loc, "Epic"));
            }
        }
    }
    let mut out: Vec<GameInstall> = Vec::new();
    for (root, store) in roots {
        for e in std::fs::read_dir(root.join("Products")).into_iter().flatten().flatten() {
            let folder = e.path();
            if folder.join("EliteDangerous64.exe").is_file() && !out.iter().any(|i| same_path(&i.folder, &folder)) {
                let shader_mod = shader_mod(&folder);
                out.push(GameInstall { folder, store, shader_mod });
            }
        }
    }
    // The current game first.
    out.sort_by_key(|i| !i.is_live());
    out
}

fn same_path(a: &Path, b: &Path) -> bool {
    a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy())
}

/// The install folder in an Epic Games Launcher manifest, if it is Elite Dangerous.
fn epic_location(manifest: &str) -> Option<PathBuf> {
    #[derive(serde::Deserialize, Default)]
    #[serde(default, rename_all = "PascalCase")]
    struct Item {
        display_name: String,
        install_location: String,
    }
    let item: Item = serde_json::from_str(manifest).ok()?;
    (item.display_name.to_ascii_lowercase().contains("elite dangerous") && !item.install_location.is_empty()).then(|| PathBuf::from(item.install_location))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_epic_manifests() {
        let m = r#"{"FormatVersion":0,"DisplayName":"Elite Dangerous","InstallLocation":"C:\\Program Files\\Epic Games\\EliteDangerous","AppName":"9c203b6ed35846e8a4a9ff1e314f6593"}"#;
        assert_eq!(epic_location(m), Some(PathBuf::from(r"C:\Program Files\Epic Games\EliteDangerous")));
        assert_eq!(epic_location(r#"{"DisplayName":"Fortnite","InstallLocation":"C:\\F"}"#), None);
        assert_eq!(epic_location("not json"), None);
    }

    #[test]
    fn spots_shader_mods() {
        let dir = std::env::temp_dir().join(format!("nexus-hud-install-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(shader_mod(&dir), None);
        std::fs::write(dir.join("d3dx.ini"), "[Constants]\n").unwrap();
        assert_eq!(shader_mod(&dir), Some(ShaderMod { edhm: false, enabled: false }));
        std::fs::create_dir_all(dir.join("EDHM-ini")).unwrap();
        std::fs::write(dir.join("d3d11.dll"), b"").unwrap();
        assert_eq!(shader_mod(&dir), Some(ShaderMod { edhm: true, enabled: true }));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
