//! Locating the game's folders (journals, Options, Steam libraries) and enumerating journal files in order.
//!
//! On Linux the game runs in a Wine prefix (Steam's Proton, Lutris, Heroic), and on macOS in one too (NotProton's
//! Steam Play, which keeps Proton's `compatdata` layout; CrossOver; Whisky). Its Windows user folder is
//! `<prefix>/drive_c/users/<name>`, and the journals and Options sit inside it just as on Windows.

use crate::time::Timestamp;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Elite Dangerous' Steam app id: Proton keeps the game's prefix in `steamapps/compatdata/359320/pfx`.
pub const STEAM_APP_ID: &str = "359320";
/// The journal folder inside a Windows user folder.
const JOURNAL_IN_PROFILE: [&str; 3] = ["Saved Games", "Frontier Developments", "Elite Dangerous"];
/// The game's Options folder inside `%LOCALAPPDATA%`.
const OPTIONS_IN_LOCAL: [&str; 3] = ["Frontier Developments", "Elite Dangerous", "Options"];

/// The journal folder the app reads, when it has said so (see [`set_journal_dir_in_use`]).
static JOURNAL_DIR_IN_USE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// `%USERPROFILE%\Saved Games\Frontier Developments\Elite Dangerous` (honouring folder redirection).
///
/// On Linux, the same folder in the game's Wine prefix: Proton's in every Steam library (native, Flatpak and
/// Snap Steam), Lutris's and Heroic's usual folders, `~/.wine`. On macOS: the prefix in every Steam library
/// (NotProton), CrossOver's bottles, `~/.wine`, and only when none of those holds a journal folder, Whisky's
/// bottles (they sit in Whisky's sandbox, and looking there makes macOS ask for access to another app's data).
/// With several, the one written to last.
pub fn default_journal_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(p) = known_folders::get_known_folder_path(known_folders::KnownFolder::SavedGames) {
            return Some(p.join("Frontier Developments").join("Elite Dangerous"));
        }
        let home = std::env::var_os("USERPROFILE")?;
        Some(JOURNAL_IN_PROFILE.iter().fold(PathBuf::from(home), |p, c| p.join(c)))
    }
    #[cfg(not(windows))]
    {
        let newest = |prefixes: Vec<PathBuf>| {
            let found = prefixes.iter().flat_map(|p| journal_dirs_in_prefix(p)).collect::<Vec<_>>();
            found.into_iter().max_by_key(|d| std::fs::metadata(d).and_then(|m| m.modified()).ok())
        };
        newest(wine_prefixes()).or_else(|| newest(sandboxed_wine_prefixes()))
    }
}

/// Tells [`options_dir`] which journal folder the app reads: a custom one may sit in a Wine prefix of its own.
pub fn set_journal_dir_in_use(dir: Option<&Path>) {
    if let Ok(mut d) = JOURNAL_DIR_IN_USE.lock() {
        *d = dir.map(Path::to_path_buf);
    }
}

/// The game's Options folder (control bindings, graphics overrides):
/// `%LOCALAPPDATA%\Frontier Developments\Elite Dangerous\Options`.
///
/// On Linux, the same folder in the Wine prefix that holds the journal folder in use, else in the prefix
/// [`default_journal_dir`] finds.
pub fn options_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let base = std::env::var_os("LOCALAPPDATA")?;
        Some(OPTIONS_IN_LOCAL.iter().fold(PathBuf::from(base), |p, c| p.join(c)))
    }
    #[cfg(not(windows))]
    {
        wine_journal_dir().and_then(|d| options_dir_beside(&d))
    }
}

/// Linux and macOS: the `drive_c` folder of the game's Wine prefix (the one holding the journal folder in use, else
/// the one found), where a Frontier or Epic launcher run under Wine installs the game. Windows: `None`.
pub fn wine_drive_c() -> Option<PathBuf> {
    if cfg!(windows) {
        return None;
    }
    // <drive_c>/users/<user>/Saved Games/Frontier Developments/Elite Dangerous
    let journals = wine_journal_dir()?;
    let drive_c = journals.ancestors().nth(5)?;
    drive_c.file_name().is_some_and(|n| n.eq_ignore_ascii_case("drive_c")).then(|| drive_c.to_path_buf())
}

/// The journal folder in use if it sits in a Wine user folder, else the one found automatically.
fn wine_journal_dir() -> Option<PathBuf> {
    let in_use = JOURNAL_DIR_IN_USE.lock().ok().and_then(|d| d.clone());
    in_use.filter(|d| options_dir_beside(d).is_some()).or_else(default_journal_dir)
}

/// `<user>/AppData/Local/Frontier Developments/Elite Dangerous/Options` for a journal folder at
/// `<user>/Saved Games/Frontier Developments/Elite Dangerous`; `None` for a journal folder anywhere else.
pub fn options_dir_beside(journal_dir: &Path) -> Option<PathBuf> {
    let mut profile = journal_dir;
    for name in JOURNAL_IN_PROFILE.iter().rev() {
        if !profile.file_name()?.to_string_lossy().eq_ignore_ascii_case(name) {
            return None;
        }
        profile = profile.parent()?;
    }
    Some(OPTIONS_IN_LOCAL.iter().fold(profile.join("AppData").join("Local"), |p, c| p.join(c)))
}

/// `%LOCALAPPDATA%\Frontier Developments\Elite Dangerous\Options\Bindings` (on Linux and macOS, in the game's Wine
/// prefix).
pub fn default_bindings_dir() -> Option<PathBuf> {
    Some(options_dir()?.join("Bindings"))
}

/// Every Steam library folder on this PC (each holds `steamapps`), from Steam's `libraryfolders.vdf`.
/// Windows: Steam in Program Files. Linux: native Steam, Flatpak Steam and Snap Steam. macOS: Steam in
/// `~/Library/Application Support` (where NotProton keeps Windows games and their prefixes).
pub fn steam_libraries() -> Vec<PathBuf> {
    let mut out: Vec<(PathBuf, PathBuf)> = Vec::new();
    for root in steam_roots() {
        let listed = std::fs::read_to_string(root.join("steamapps").join("libraryfolders.vdf")).map(|t| parse_library_folders(&t)).unwrap_or_default();
        for lib in std::iter::once(root).chain(listed) {
            if !lib.join("steamapps").is_dir() {
                continue;
            }
            // ~/.steam/steam is usually a link to ~/.local/share/Steam: compare where they lead.
            let key = std::fs::canonicalize(&lib).unwrap_or_else(|_| lib.clone());
            if !out.iter().any(|(k, _)| same_path(k, &key)) {
                out.push((key, lib));
            }
        }
    }
    out.into_iter().map(|(_, lib)| lib).collect()
}

/// The library folders listed in Steam's `libraryfolders.vdf` (`"path"  "D:\\SteamLibrary"`).
pub fn parse_library_folders(vdf: &str) -> Vec<PathBuf> {
    vdf.lines()
        .filter_map(|l| {
            let rest = l.trim().strip_prefix("\"path\"")?.trim();
            let v = rest.strip_prefix('"')?.strip_suffix('"')?;
            Some(PathBuf::from(v.replace("\\\\", "\\")))
        })
        .collect()
}

#[cfg(windows)]
fn steam_roots() -> Vec<PathBuf> {
    ["ProgramFiles(x86)", "ProgramFiles"].iter().filter_map(std::env::var_os).map(|pf| PathBuf::from(pf).join("Steam")).collect()
}

#[cfg(not(windows))]
fn steam_roots() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else { return Vec::new() };
    let mut roots: Vec<PathBuf> = std::env::var_os("XDG_DATA_HOME").map(|x| PathBuf::from(x).join("Steam")).into_iter().collect();
    for rel in [".local/share/Steam", ".steam/steam", ".steam/root", ".var/app/com.valvesoftware.Steam/.local/share/Steam", ".var/app/com.valvesoftware.Steam/data/Steam", "snap/steam/common/.local/share/Steam"] {
        roots.push(home.join(rel));
    }
    if cfg!(target_os = "macos") {
        roots.push(home.join(MACOS_STEAM));
    }
    roots
}

/// The macOS Steam client's folder under `$HOME`, with its own `steamapps` (the default library).
#[cfg(not(windows))]
const MACOS_STEAM: &str = "Library/Application Support/Steam";
/// CrossOver's bottles under `$HOME` (each bottle is a Wine prefix).
#[cfg(not(windows))]
const CROSSOVER_BOTTLES: &str = "Library/Application Support/CrossOver/Bottles";
/// Whisky's bottles under `$HOME`, inside Whisky's sandbox container.
#[cfg(not(windows))]
const WHISKY_BOTTLES: &str = "Library/Containers/com.isaacmarovitz.Whisky/Bottles";

fn same_path(a: &Path, b: &Path) -> bool {
    if cfg!(windows) { a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy()) } else { a == b }
}

/// Wine prefixes that may hold the game: Proton's in every Steam library (NotProton's too, on macOS), Lutris's
/// (`~/Games/<game>`), Heroic's (`~/Games/Heroic/Prefixes/default/<game>`), `~/.wine` and, on macOS, CrossOver's
/// bottles.
#[cfg(not(windows))]
fn wine_prefixes() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = steam_libraries().iter().map(|l| l.join("steamapps").join("compatdata").join(STEAM_APP_ID).join("pfx")).collect();
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        out.push(home.join(".wine"));
        let mut parents = vec![home.join("Games"), home.join("Games/Heroic/Prefixes/default"), home.join("Games/Heroic/Prefixes")];
        if cfg!(target_os = "macos") {
            parents.push(home.join(CROSSOVER_BOTTLES));
        }
        for parent in parents {
            out.extend(subfolders(&parent));
        }
    }
    out
}

/// macOS: Whisky's bottles, looked at only when no other prefix holds the journals. They sit in another app's
/// sandbox container, and macOS asks the user before letting Elite Nexus look there. Elsewhere: none.
#[cfg(not(windows))]
fn sandboxed_wine_prefixes() -> Vec<PathBuf> {
    match std::env::var_os("HOME").map(PathBuf::from) {
        Some(home) if cfg!(target_os = "macos") => subfolders(&home.join(WHISKY_BOTTLES)),
        _ => Vec::new(),
    }
}

#[cfg(not(windows))]
fn subfolders(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir).into_iter().flatten().flatten().map(|e| e.path()).collect()
}

/// The journal folders in a Wine prefix: one per Windows user that has played (Proton's is `steamuser`).
#[cfg(not(windows))]
fn journal_dirs_in_prefix(prefix: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(prefix.join("drive_c").join("users"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|user| JOURNAL_IN_PROFILE.iter().fold(user.path(), |p, c| p.join(c)))
        .filter(|d| d.is_dir())
        .collect()
}

/// A journal file with the session start time and part number decoded from its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalFile {
    pub path: PathBuf,
    pub file_name: String,
    /// Session start time encoded in the file name (local time of the game client, treated as UTC).
    pub started: Timestamp,
    pub part: u32,
    pub size: u64,
}

/// Decodes `Journal.2026-10-12T073031.01.log` (current) and `Journal.170404170616.01.log` (pre-2021).
pub fn parse_journal_file_name(name: &str) -> Option<(Timestamp, u32)> {
    let rest = name.strip_prefix("Journal.")?.strip_suffix(".log")?;
    let (stamp, part) = rest.rsplit_once('.')?;
    let part: u32 = part.parse().ok()?;
    let ts = if stamp.len() == 17 && stamp.as_bytes()[10] == b'T' {
        // 2026-10-12T073031
        let iso = format!("{}T{}:{}:{}Z", &stamp[..10], &stamp[11..13], &stamp[13..15], &stamp[15..17]);
        Timestamp::parse(&iso)?
    } else if stamp.len() == 12 && stamp.bytes().all(|b| b.is_ascii_digit()) {
        // 170404170616 -> 2017-04-04T17:06:16
        let iso = format!(
            "20{}-{}-{}T{}:{}:{}Z",
            &stamp[0..2],
            &stamp[2..4],
            &stamp[4..6],
            &stamp[6..8],
            &stamp[8..10],
            &stamp[10..12]
        );
        Timestamp::parse(&iso)?
    } else {
        return None;
    };
    Some((ts, part))
}

/// All `Journal.*.log` files in `dir`, oldest first.
pub fn list_journal_files(dir: &Path) -> std::io::Result<Vec<JournalFile>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let Some((started, part)) = parse_journal_file_name(&file_name) else { continue };
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        out.push(JournalFile { path: entry.path(), file_name, started, part, size });
    }
    out.sort_by(|a, b| (a.started, a.part, &a.file_name).cmp(&(b.started, b.part, &b.file_name)));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_both_file_name_styles() {
        let (t, p) = parse_journal_file_name("Journal.2026-10-12T073031.01.log").unwrap();
        assert_eq!(t.to_iso(), "2026-10-12T07:30:31Z");
        assert_eq!(p, 1);
        let (t, p) = parse_journal_file_name("Journal.170404170616.02.log").unwrap();
        assert_eq!(t.to_iso(), "2017-04-04T17:06:16Z");
        assert_eq!(p, 2);
        assert!(parse_journal_file_name("Status.json").is_none());
    }

    #[test]
    fn reads_steam_library_folders() {
        let vdf = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"C:\\\\Program Files (x86)\\\\Steam\"\n\t\t\"label\"\t\t\"\"\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n\t}\n}";
        assert_eq!(parse_library_folders(vdf), vec![PathBuf::from(r"C:\Program Files (x86)\Steam"), PathBuf::from(r"D:\SteamLibrary")]);
        let linux = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"/home/cmdr/.local/share/Steam\"\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"/mnt/games/SteamLibrary\"\n\t}\n}";
        assert_eq!(parse_library_folders(linux), vec![PathBuf::from("/home/cmdr/.local/share/Steam"), PathBuf::from("/mnt/games/SteamLibrary")]);
    }

    #[test]
    fn finds_options_next_to_the_journals() {
        let pfx = PathBuf::from("steamapps").join("compatdata").join(STEAM_APP_ID).join("pfx").join("drive_c").join("users").join("steamuser");
        let journals = pfx.join("Saved Games").join("Frontier Developments").join("Elite Dangerous");
        let options = pfx.join("AppData").join("Local").join("Frontier Developments").join("Elite Dangerous").join("Options");
        assert_eq!(options_dir_beside(&journals), Some(options));
        assert_eq!(options_dir_beside(&PathBuf::from("replays").join("Elite Dangerous")), None);
    }

    /// NotProton (Steam Play on macOS) keeps Proton's layout inside the macOS Steam library; CrossOver bottles and
    /// prefixes made before NotProton's `steamuser` layout name the Windows user `crossover`.
    #[cfg(not(windows))]
    #[test]
    fn finds_journals_in_a_notproton_prefix() {
        let root = std::env::temp_dir().join(format!("nexus-journal-paths-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let pfx = root.join(MACOS_STEAM).join("steamapps").join("compatdata").join(STEAM_APP_ID).join("pfx");
        let journals = |user: &str| JOURNAL_IN_PROFILE.iter().fold(pfx.join("drive_c").join("users").join(user), |p, c| p.join(c));
        std::fs::create_dir_all(journals("steamuser")).unwrap();
        std::fs::create_dir_all(pfx.join("drive_c").join("users").join("Public")).unwrap();
        assert_eq!(journal_dirs_in_prefix(&pfx), vec![journals("steamuser")]);
        let options = pfx.join("drive_c/users/steamuser/AppData/Local/Frontier Developments/Elite Dangerous/Options");
        assert_eq!(options_dir_beside(&journals("steamuser")), Some(options));
        std::fs::create_dir_all(journals("crossover")).unwrap();
        let mut both = journal_dirs_in_prefix(&pfx);
        both.sort();
        assert_eq!(both, vec![journals("crossover"), journals("steamuser")]);
        let _ = std::fs::remove_dir_all(&root);
    }
}
