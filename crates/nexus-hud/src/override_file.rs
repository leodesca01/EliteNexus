//! `GraphicsConfigurationOverride.xml`, the file in which the game looks for a commander's own
//! graphics values (HUD colours in `<GUIColour><Default>`, tone mapping in `<HDRNode_Reference>`)
//! on top of the `GraphicsConfiguration.xml` it ships with.
//!
//! Edits are surgical: only the values asked for are written, everything else in the file (other
//! overrides, comments, layout) stays byte for byte. A malformed file is never rewritten, and the
//! commander's own file is copied aside once before Elite Nexus first changes it.

use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::error::HudError;
use crate::matrix::HudMatrix;

pub const FILE_NAME: &str = "GraphicsConfigurationOverride.xml";
/// The commander's file as it was before Elite Nexus first changed it.
pub const BACKUP_NAME: &str = "GraphicsConfigurationOverride.before-elite-nexus.xml";

/// `%LOCALAPPDATA%\Frontier Developments\Elite Dangerous\Options\Graphics`; on Linux and macOS, in the game's Wine prefix.
pub fn default_graphics_dir() -> Option<PathBuf> {
    Some(nexus_journal::options_dir()?.join("Graphics"))
}

/// The override file as found on disk.
#[derive(Debug, Clone)]
pub struct OverrideFile {
    pub path: PathBuf,
    /// `None` when the file doesn't exist (the game then uses its standard colours).
    pub text: Option<String>,
}

impl OverrideFile {
    /// Reads `FILE_NAME` in `dir`. A missing file is not an error.
    pub fn load(dir: &Path) -> Result<Self, HudError> {
        let path = dir.join(FILE_NAME);
        match std::fs::read(&path) {
            Ok(bytes) => Ok(OverrideFile { text: Some(String::from_utf8_lossy(&bytes).into_owned()), path }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(OverrideFile { path, text: None }),
            Err(source) => Err(HudError::Read { path, source }),
        }
    }

    /// The matrix the game will use: `None` for the standard colours.
    pub fn matrix(&self) -> Result<Option<HudMatrix>, HudError> {
        match &self.text {
            Some(t) => read_matrix(t),
            None => Ok(None),
        }
    }

    pub fn backup_path(&self) -> PathBuf {
        self.path.with_file_name(BACKUP_NAME)
    }

    /// Writes `matrix` (`None` = back to the standard colours).
    pub fn save(&mut self, matrix: Option<&HudMatrix>) -> Result<(), HudError> {
        let new = with_matrix(self.text.as_deref(), matrix)?;
        self.write(new)
    }

    /// Replaces the file with `new`, the current text edited (see [`with_matrix`], [`with_value`]).
    /// Copies the commander's original file aside first, once; replaces the file atomically.
    pub fn write(&mut self, new: String) -> Result<(), HudError> {
        if self.text.as_deref().unwrap_or("") == new {
            return Ok(());
        }
        let werr = |path: &Path| {
            let path = path.to_path_buf();
            move |source| HudError::Write { path, source }
        };
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(werr(dir))?;
        }
        let backup = self.backup_path();
        if self.text.is_some() && !backup.exists() {
            std::fs::copy(&self.path, &backup).map_err(werr(&backup))?;
        }
        let tmp = self.path.with_extension("xml.elite-nexus-tmp");
        std::fs::write(&tmp, &new).map_err(werr(&tmp))?;
        std::fs::rename(&tmp, &self.path).map_err(werr(&self.path))?;
        tracing::info!(path = %self.path.display(), "graphics override written");
        self.text = Some(new);
        Ok(())
    }
}

fn strip_bom(text: &str) -> (&str, &str) {
    match text.strip_prefix('\u{feff}') {
        Some(rest) => ("\u{feff}", rest),
        None => ("", text),
    }
}

fn parse(text: &str) -> Result<roxmltree::Document<'_>, HudError> {
    let doc = roxmltree::Document::parse(text)?;
    let root = doc.root_element();
    if root.tag_name().name() != "GraphicsConfig" {
        return Err(HudError::NotGraphicsConfig(root.tag_name().name().to_string()));
    }
    Ok(doc)
}

fn child<'a, 'i>(node: roxmltree::Node<'a, 'i>, name: &str) -> Option<roxmltree::Node<'a, 'i>> {
    node.children().find(|n| n.is_element() && n.tag_name().name() == name)
}

/// The `<GUIColour><Default>` matrix in an override file's text. `None` when the file sets no
/// colours (the game then shows the standard orange). Rows the file leaves out keep the standard
/// values, as the game reads them from its own configuration.
pub fn read_matrix(text: &str) -> Result<Option<HudMatrix>, HudError> {
    let (_, body) = strip_bom(text);
    if body.trim().is_empty() {
        return Ok(None);
    }
    let doc = parse(body)?;
    let Some(default) = child(doc.root_element(), "GUIColour").and_then(|g| child(g, "Default")) else { return Ok(None) };
    let mut m = HudMatrix::IDENTITY;
    let mut any = false;
    for (i, row) in HudMatrix::ROW_NAMES.iter().enumerate() {
        if let Some(el) = child(default, row) {
            let text = el.text().unwrap_or("");
            *m.row_mut(i) = HudMatrix::parse_row(text).ok_or_else(|| HudError::BadRow { row, text: text.trim().to_string() })?;
            any = true;
        }
    }
    Ok(any.then_some(m))
}

/// The override file's text with `matrix` written into it (`None` removes the colour override).
/// `text` is the current file, `None` or blank when there is none.
pub fn with_matrix(text: Option<&str>, matrix: Option<&HudMatrix>) -> Result<String, HudError> {
    let (bom, body) = strip_bom(text.unwrap_or(""));
    if body.trim().is_empty() {
        return Ok(match matrix {
            Some(m) => format!("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<GraphicsConfig>\n{}\n</GraphicsConfig>\n", gui_colour_block(m, "\t", "\n")),
            None => text.unwrap_or("").to_string(),
        });
    }
    let doc = parse(body)?;
    let nl = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let root = doc.root_element();
    let gui = child(root, "GUIColour");
    let default = gui.and_then(|g| child(g, "Default"));
    let mut edits: Vec<(Range<usize>, String)> = Vec::new();
    match (matrix, gui, default) {
        (None, None, _) | (None, Some(_), None) => return Ok(text.unwrap_or("").to_string()),
        (None, Some(gui), Some(default)) => {
            // Drop the colours; drop <GUIColour> too when nothing else is left in it.
            let alone = gui.children().filter(|n| n.is_element()).count() == 1;
            let node = if alone { gui } else { default };
            edits.push((whole_lines(body, node.range()), String::new()));
        }
        (Some(m), _, Some(default)) => {
            let ind = indent_at(body, default.range().start);
            // New rows line up with the existing ones.
            let child_ind = default.children().find(|n| n.is_element()).map(|c| indent_at(body, c.range().start)).filter(|s| !s.is_empty()).unwrap_or_else(|| format!("{ind}\t"));
            let mut missing = String::new();
            for (i, row) in HudMatrix::ROW_NAMES.iter().enumerate() {
                let value = HudMatrix::format_row(m.rows()[i]);
                match child(default, row) {
                    Some(el) => match el.first_child().filter(|t| t.is_text()) {
                        Some(t) if el.children().count() == 1 => edits.push((t.range(), value)),
                        _ => edits.push((el.range(), format!("<{row}>{value}</{row}>"))),
                    },
                    None => missing.push_str(&format!("{child_ind}<{row}>{value}</{row}>{nl}")),
                }
            }
            if !missing.is_empty() {
                edits.push(insert_into(body, default, &missing, &ind, nl));
            }
        }
        (Some(m), Some(gui), None) => {
            let ind = format!("{}\t", indent_at(body, gui.range().start));
            let block = format!("{ind}{}{nl}", default_block(m, &ind, nl));
            edits.push(insert_into(body, gui, &block, &indent_at(body, gui.range().start), nl));
        }
        (Some(m), None, _) => {
            let block = format!("{}{nl}", gui_colour_block(m, "\t", nl));
            edits.push(insert_into(body, root, &block, "", nl));
        }
    }
    let mut out = body.to_string();
    edits.sort_by_key(|(r, _)| std::cmp::Reverse(r.start));
    for (range, with) in edits {
        out.replace_range(range, &with);
    }
    Ok(format!("{bom}{out}"))
}

/// The text of the element at `path` below `<GraphicsConfig>`, trimmed (for example
/// `["HDRNode_Reference", "GlareCompensation"]`). `None` when the file doesn't set it.
pub fn read_value(text: &str, path: &[&str]) -> Result<Option<String>, HudError> {
    let (_, body) = strip_bom(text);
    if body.trim().is_empty() {
        return Ok(None);
    }
    let doc = parse(body)?;
    let mut node = doc.root_element();
    for name in path {
        match child(node, name) {
            Some(c) => node = c,
            None => return Ok(None),
        }
    }
    Ok(Some(node.text().unwrap_or("").trim().to_string()))
}

/// The override file's text with the element at `path` set to `value`, creating it (and any
/// missing parents) when needed, or removed with `None` (parents it leaves empty go too). `text`
/// is the current file, `None` or blank when there is none.
pub fn with_value(text: Option<&str>, path: &[&str], value: Option<&str>) -> Result<String, HudError> {
    let unchanged = || text.unwrap_or("").to_string();
    if path.is_empty() {
        return Ok(unchanged());
    }
    let value = value.map(escape);
    let (bom, body) = strip_bom(text.unwrap_or(""));
    if body.trim().is_empty() {
        return Ok(match value {
            Some(v) => format!("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<GraphicsConfig>\n{}</GraphicsConfig>\n", nested_block(path, &v, "\t", "\n")),
            None => unchanged(),
        });
    }
    let doc = parse(body)?;
    let nl = if body.contains("\r\n") { "\r\n" } else { "\n" };
    // The deepest element of `path` that exists, and how much of the path that covers.
    let mut node = doc.root_element();
    let mut depth = 0;
    while depth < path.len() {
        match child(node, path[depth]) {
            Some(c) => {
                node = c;
                depth += 1;
            }
            None => break,
        }
    }
    let edit = match (value, depth == path.len()) {
        (None, false) => return Ok(unchanged()),
        (None, true) => (whole_lines(body, removal_root(node).range()), String::new()),
        (Some(v), true) => match node.first_child().filter(|t| t.is_text()) {
            Some(t) if node.children().count() == 1 => (t.range(), v),
            _ => {
                let name = node.tag_name().name();
                (node.range(), format!("<{name}>{v}</{name}>"))
            }
        },
        (Some(v), false) => {
            let ind = indent_at(body, node.range().start);
            let child_ind = node.children().find(|n| n.is_element()).map(|c| indent_at(body, c.range().start)).filter(|s| !s.is_empty()).unwrap_or_else(|| format!("{ind}\t"));
            insert_into(body, node, &nested_block(&path[depth..], &v, &child_ind, nl), &ind, nl)
        }
    };
    let mut out = body.to_string();
    out.replace_range(edit.0, &edit.1);
    Ok(format!("{bom}{out}"))
}

/// `names` nested, the innermost holding `value`, as complete lines starting at indent `ind`.
fn nested_block(names: &[&str], value: &str, ind: &str, nl: &str) -> String {
    let last = names.len() - 1;
    let mut s = String::new();
    for (k, name) in names[..last].iter().enumerate() {
        s.push_str(&format!("{ind}{}<{name}>{nl}", "\t".repeat(k)));
    }
    s.push_str(&format!("{ind}{}<{n}>{value}</{n}>{nl}", "\t".repeat(last), n = names[last]));
    for (k, name) in names[..last].iter().enumerate().rev() {
        s.push_str(&format!("{ind}{}</{name}>{nl}", "\t".repeat(k)));
    }
    s
}

/// What to delete when removing `leaf`: the highest ancestor below `<GraphicsConfig>` that holds
/// nothing else (no other elements, text or comments).
fn removal_root<'a, 'i>(leaf: roxmltree::Node<'a, 'i>) -> roxmltree::Node<'a, 'i> {
    let mut n = leaf;
    while let Some(p) = n.parent_element() {
        if p.parent_element().is_none() {
            break;
        }
        let alone = p.children().all(|c| c == n || (c.is_text() && c.text().unwrap_or("").trim().is_empty()));
        if !alone {
            break;
        }
        n = p;
    }
    n
}

fn escape(v: &str) -> String {
    v.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// An edit that puts `lines` (complete lines, each ending in `nl`) just before `node`'s end tag,
/// or turns a self-closing `<Node />` into `<Node>…</Node>`. `ind` is the node's own indent.
fn insert_into(body: &str, node: roxmltree::Node<'_, '_>, lines: &str, ind: &str, nl: &str) -> (Range<usize>, String) {
    let range = node.range();
    let own = &body[range.clone()];
    let name = node.tag_name().name();
    if own.ends_with("/>") {
        return (range, format!("<{name}>{nl}{lines}{ind}</{name}>"));
    }
    let close = range.start + own.rfind("</").unwrap_or(own.len());
    // Start a fresh line when the end tag shares one with other content.
    let line_start = body[..close].rfind('\n').map_or(0, |i| i + 1);
    if body[line_start..close].trim().is_empty() {
        (line_start..line_start, lines.to_string())
    } else {
        (close..close, format!("{nl}{lines}{ind}"))
    }
}

/// `range` widened to whole lines when nothing else shares them (so removing it leaves no blank line).
fn whole_lines(body: &str, range: Range<usize>) -> Range<usize> {
    let line_start = body[..range.start].rfind('\n').map_or(0, |i| i + 1);
    if !body[line_start..range.start].trim().is_empty() {
        return range;
    }
    let rest = &body[range.end..];
    let trailing = rest.len() - rest.trim_start_matches([' ', '\t']).len();
    let after = &rest[trailing..];
    let eol = if after.starts_with("\r\n") {
        2
    } else if after.starts_with('\n') {
        1
    } else if after.is_empty() {
        0
    } else {
        return range;
    };
    line_start..range.end + trailing + eol
}

/// The whitespace before `pos` on its line.
fn indent_at(body: &str, pos: usize) -> String {
    let line_start = body[..pos].rfind('\n').map_or(0, |i| i + 1);
    let lead = &body[line_start..pos];
    if lead.trim().is_empty() { lead.to_string() } else { String::new() }
}

/// `<Default>…</Default>`; the first line carries no indent (the caller places it), later lines
/// are indented from `ind`.
fn default_block(m: &HudMatrix, ind: &str, nl: &str) -> String {
    let mut s = format!("<Default>{nl}{ind}\t<LocalisationName>Standard</LocalisationName>{nl}");
    for (i, row) in HudMatrix::ROW_NAMES.iter().enumerate() {
        s.push_str(&format!("{ind}\t<{row}>{}</{row}>{nl}", HudMatrix::format_row(m.rows()[i])));
    }
    s.push_str(&format!("{ind}</Default>"));
    s
}

/// `<GUIColour>` with the matrix, indented one level from `ind`, the first line included.
fn gui_colour_block(m: &HudMatrix, ind: &str, nl: &str) -> String {
    format!("{ind}<GUIColour>{nl}{ind}\t{}{nl}{ind}</GUIColour>", default_block(m, &format!("{ind}\t"), nl))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLUE: HudMatrix = HudMatrix { red: [0.0, 0.39, 1.0], green: [0.0, 1.0, 0.0], blue: [1.0, 0.0, 0.0] };

    fn round_trip(text: Option<&str>) -> String {
        let out = with_matrix(text, Some(&BLUE)).unwrap();
        assert_eq!(read_matrix(&out).unwrap(), Some(BLUE), "{out}");
        out
    }

    #[test]
    fn creates_a_file_from_nothing() {
        let out = round_trip(None);
        assert!(out.starts_with("<?xml"));
        assert_eq!(round_trip(Some("  \n")), out);
        // Removing colours from nothing changes nothing.
        assert_eq!(with_matrix(None, None).unwrap(), "");
    }

    #[test]
    fn fills_the_self_closing_root_the_game_writes() {
        let text = "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<GraphicsConfig />\n";
        let out = round_trip(Some(text));
        assert_eq!(
            out,
            "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<GraphicsConfig>\n\t<GUIColour>\n\t\t<Default>\n\t\t\t<LocalisationName>Standard</LocalisationName>\n\t\t\t<MatrixRed>0, 0.39, 1</MatrixRed>\n\t\t\t<MatrixGreen>0, 1, 0</MatrixGreen>\n\t\t\t<MatrixBlue>1, 0, 0</MatrixBlue>\n\t\t</Default>\n\t</GUIColour>\n</GraphicsConfig>\n"
        );
        // And back: the colours go, the rest stays.
        let back = with_matrix(Some(&out), None).unwrap();
        assert_eq!(back, "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<GraphicsConfig>\n</GraphicsConfig>\n");
        assert_eq!(read_matrix(&back).unwrap(), None);
    }

    #[test]
    fn keeps_everything_else_in_the_file() {
        let text = "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\r\n<!-- mine -->\r\n<GraphicsConfig>\r\n\t<GalaxyMap>\r\n\t\t<HighSpec>\r\n\t\t\t<StarFieldNum>4000</StarFieldNum>\r\n\t\t</HighSpec>\r\n\t</GalaxyMap>\r\n</GraphicsConfig>\r\n";
        let out = round_trip(Some(text));
        assert!(out.starts_with('\u{feff}'));
        assert!(out.contains("<!-- mine -->") && out.contains("<StarFieldNum>4000</StarFieldNum>"));
        assert!(out.contains("\t<GUIColour>\r\n\t\t<Default>\r\n"), "{out:?}");
        assert!(!out.replace("\r\n", "").contains('\n'), "mixed line endings: {out:?}");
        assert_eq!(with_matrix(Some(&out), None).unwrap(), text);
    }

    #[test]
    fn edits_only_the_rows() {
        let text = "<GraphicsConfig>\n  <GUIColour>\n    <Default>\n      <LocalisationName>Mine</LocalisationName>\n      <MatrixRed>   1, 0, 0 </MatrixRed>\n      <MatrixGreen>0, 1, 0</MatrixGreen>\n    </Default>\n    <Other><MatrixRed>1,1,1</MatrixRed></Other>\n  </GUIColour>\n</GraphicsConfig>";
        // A missing row reads as the standard one.
        assert_eq!(read_matrix(text).unwrap(), Some(HudMatrix::IDENTITY));
        let out = round_trip(Some(text));
        assert!(out.contains("<LocalisationName>Mine</LocalisationName>"));
        assert!(out.contains("      <MatrixRed>0, 0.39, 1</MatrixRed>\n"));
        assert!(out.contains("      <MatrixBlue>1, 0, 0</MatrixBlue>\n    </Default>"), "{out}");
        assert!(out.contains("<Other><MatrixRed>1,1,1</MatrixRed></Other>"));
        // Removing keeps <GUIColour> for the other entry.
        let back = with_matrix(Some(&out), None).unwrap();
        assert!(back.contains("<GUIColour>\n    <Other>"), "{back}");
        assert_eq!(read_matrix(&back).unwrap(), None);
    }

    #[test]
    fn adds_default_to_an_existing_gui_colour() {
        let out = round_trip(Some("<GraphicsConfig><GUIColour/></GraphicsConfig>"));
        assert!(out.contains("<GUIColour>\n\t<Default>"), "{out}");
        round_trip(Some("<GraphicsConfig>\n\t<GUIColour>\n\t</GUIColour>\n</GraphicsConfig>"));
        round_trip(Some("<GraphicsConfig><GUIColour><Default/></GUIColour></GraphicsConfig>"));
        round_trip(Some("<GraphicsConfig><GUIColour><Default><MatrixRed/></Default></GUIColour></GraphicsConfig>"));
    }

    #[test]
    fn refuses_files_the_game_could_not_read() {
        assert!(matches!(with_matrix(Some("<GraphicsConfig><GUIColour>"), Some(&BLUE)), Err(HudError::Xml(_))));
        assert!(matches!(with_matrix(Some("<Root/>"), Some(&BLUE)), Err(HudError::NotGraphicsConfig(_))));
        assert!(matches!(
            read_matrix("<GraphicsConfig><GUIColour><Default><MatrixRed>1, 0</MatrixRed></Default></GUIColour></GraphicsConfig>"),
            Err(HudError::BadRow { row: "MatrixRed", .. })
        ));
    }

    const GLARE: &[&str] = &["HDRNode_Reference", "GlareCompensation"];

    #[test]
    fn values_are_created_with_their_parents_and_removed_with_them() {
        let text = "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\r\n<GraphicsConfig />\r\n";
        let out = with_value(Some(text), GLARE, Some("1.0")).unwrap();
        assert_eq!(out, "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\r\n<GraphicsConfig>\r\n\t<HDRNode_Reference>\r\n\t\t<GlareCompensation>1.0</GlareCompensation>\r\n\t</HDRNode_Reference>\r\n</GraphicsConfig>\r\n");
        assert_eq!(read_value(&out, GLARE).unwrap().as_deref(), Some("1.0"));
        // A sibling joins the existing parent.
        let two = with_value(Some(&out), &["HDRNode_Reference", "ACES_C"], Some("2.43")).unwrap();
        assert!(two.contains("\t\t<GlareCompensation>1.0</GlareCompensation>\r\n\t\t<ACES_C>2.43</ACES_C>\r\n\t</HDRNode_Reference>"), "{two:?}");
        // Changing keeps the layout; removing one keeps the parent for the other.
        let changed = with_value(Some(&two), GLARE, Some("0.8")).unwrap();
        assert_eq!(changed, two.replace(">1.0<", ">0.8<"));
        let one = with_value(Some(&changed), GLARE, None).unwrap();
        assert_eq!(read_value(&one, GLARE).unwrap(), None);
        assert!(one.contains("<HDRNode_Reference>\r\n\t\t<ACES_C>2.43</ACES_C>"), "{one:?}");
        // Removing the last one takes the empty parent with it.
        let none = with_value(Some(&one), &["HDRNode_Reference", "ACES_C"], None).unwrap();
        assert_eq!(none, "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\r\n<GraphicsConfig>\r\n</GraphicsConfig>\r\n");
        // Nothing to remove: nothing changes.
        assert_eq!(with_value(Some(text), GLARE, None).unwrap(), text);
        assert_eq!(with_value(None, GLARE, None).unwrap(), "");
    }

    #[test]
    fn values_live_alongside_the_colours() {
        let colours = with_matrix(None, Some(&BLUE)).unwrap();
        let both = with_value(Some(&colours), GLARE, Some("1.0")).unwrap();
        assert_eq!(read_matrix(&both).unwrap(), Some(BLUE));
        assert_eq!(read_value(&both, GLARE).unwrap().as_deref(), Some("1.0"));
        // Each can go without disturbing the other.
        assert_eq!(with_value(Some(&both), GLARE, None).unwrap(), colours);
        let glare_only = with_matrix(Some(&both), None).unwrap();
        assert_eq!(read_matrix(&glare_only).unwrap(), None);
        assert_eq!(read_value(&glare_only, GLARE).unwrap().as_deref(), Some("1.0"));
        // From nothing at all.
        let fresh = with_value(None, GLARE, Some("1.0")).unwrap();
        assert!(fresh.starts_with("<?xml") && read_value(&fresh, GLARE).unwrap().is_some());
    }

    #[test]
    fn values_keep_comments_and_unusual_layouts() {
        let text = "<GraphicsConfig><HDRNode_Reference><!-- mine --><GlareCompensation> 1.3 </GlareCompensation></HDRNode_Reference></GraphicsConfig>";
        assert_eq!(read_value(text, GLARE).unwrap().as_deref(), Some("1.3"));
        // The comment keeps its parent alive.
        let out = with_value(Some(text), GLARE, None).unwrap();
        assert_eq!(out, "<GraphicsConfig><HDRNode_Reference><!-- mine --></HDRNode_Reference></GraphicsConfig>");
        let out = with_value(Some("<GraphicsConfig><HDRNode_Reference><GlareCompensation/></HDRNode_Reference></GraphicsConfig>"), GLARE, Some("1")).unwrap();
        assert_eq!(out, "<GraphicsConfig><HDRNode_Reference><GlareCompensation>1</GlareCompensation></HDRNode_Reference></GraphicsConfig>");
        // Values are escaped, and malformed files refused.
        let out = with_value(None, GLARE, Some("a<b")).unwrap();
        assert!(out.contains("a&lt;b") && read_value(&out, GLARE).unwrap().as_deref() == Some("a<b"));
        assert!(matches!(with_value(Some("<GraphicsConfig>"), GLARE, Some("1")), Err(HudError::Xml(_))));
    }

    #[test]
    fn saves_atomically_with_a_backup_of_the_original() {
        let dir = std::env::temp_dir().join(format!("nexus-hud-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let original = "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<GraphicsConfig />\n";
        std::fs::write(dir.join(FILE_NAME), original).unwrap();
        let mut f = OverrideFile::load(&dir).unwrap();
        assert_eq!(f.matrix().unwrap(), None);
        f.save(Some(&BLUE)).unwrap();
        assert_eq!(OverrideFile::load(&dir).unwrap().matrix().unwrap(), Some(BLUE));
        assert_eq!(std::fs::read_to_string(dir.join(BACKUP_NAME)).unwrap(), original);
        // The backup keeps the very first version.
        f.save(Some(&HudMatrix::IDENTITY.scaled(0.5))).unwrap();
        assert_eq!(std::fs::read_to_string(dir.join(BACKUP_NAME)).unwrap(), original);
        f.save(None).unwrap();
        assert_eq!(OverrideFile::load(&dir).unwrap().matrix().unwrap(), None);
        // No file: nothing to back up, the folder is created.
        let fresh = dir.join("Options").join("Graphics");
        let mut f = OverrideFile::load(&fresh).unwrap();
        f.save(Some(&BLUE)).unwrap();
        assert!(fresh.join(FILE_NAME).exists() && !fresh.join(BACKUP_NAME).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
