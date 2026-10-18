//! Error type for reading and writing the HUD colour override.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum HudError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// The game would not read a malformed file either, so it is never rewritten.
    #[error("the file is not well-formed XML: {0}")]
    Xml(#[from] roxmltree::Error),
    #[error("the root element is <{0}>, the game expects <GraphicsConfig>")]
    NotGraphicsConfig(String),
    /// A `<MatrixRed>`-style row that is not three comma-separated numbers.
    #[error("<{row}> should hold three numbers, found {text:?}")]
    BadRow { row: &'static str, text: String },
    /// A single value such as `<GlareCompensation>` that is not a number.
    #[error("<{name}> should be a number, found {text:?}")]
    BadValue { name: &'static str, text: String },
    #[error("The game's Options folder wasn't found (on Linux or macOS: set the journal folder in Settings to the one in the game's Wine prefix)")]
    NoOptionsDir,
}
