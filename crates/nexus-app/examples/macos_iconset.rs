//! macOS: renders the app icon (drawn in code, `src/icon.rs`) as an `.iconset` folder, which `iconutil` turns into
//! the `.icns` that Elite Nexus.app shows in the Finder and the Dock. `tools/macos/build-app.sh` runs it:
//!
//! `cargo run --release -p nexus-app --example macos_iconset -- <folder>.iconset`
//!
//! The design sits the way macOS lays out app icons: on a rounded square with a margin around it (824 of 1024
//! points), so it lines up with the Dock's other icons instead of filling the whole tile.

use std::path::PathBuf;

#[allow(dead_code)]
#[path = "../src/icon.rs"]
mod icon;

/// The share of the tile the rounded square fills (Apple's app icon grid: 824 of 1024 points).
const FILL: f32 = 824.0 / 1024.0;
/// The rounded square's corner radius as a share of its side (close to Apple's 185 of 824).
const CORNER: f32 = 0.225;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::args_os().nth(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("EliteNexus.iconset"));
    std::fs::create_dir_all(&dir)?;
    for points in [16u32, 32, 128, 256, 512] {
        for scale in [1u32, 2] {
            let n = points * scale;
            let name = if scale == 1 { format!("icon_{points}x{points}.png") } else { format!("icon_{points}x{points}@2x.png") };
            image::save_buffer(dir.join(name), &macos_rgba(n), n, n, image::ColorType::Rgba8)?;
        }
    }
    println!("{}", dir.display());
    Ok(())
}

/// An `n`×`n` RGBA rendering in macOS's layout: the design's dark square scaled to [`FILL`] of the tile, its
/// corners rounded.
fn macos_rgba(n: u32) -> Vec<u8> {
    let inner = ((n as f32 * FILL).round() as u32).max(1);
    let off = (n - inner) / 2;
    // The dark square spans 60 of the design's 64 units: render the design larger so the square fills `inner`.
    let full = (inner as f32 * 64.0 / 60.0).round() as u32;
    let crop = (full - inner) / 2;
    let mut out = vec![0u8; (n * n * 4) as usize];
    for y in 0..inner {
        for x in 0..inner {
            let [r, g, b, a] = icon::pixel(full, x + crop, y + crop);
            let alpha = (a as f32 * rounded_cover(inner, x, y)).round() as u8;
            let i = (((y + off) * n + x + off) * 4) as usize;
            out[i..i + 4].copy_from_slice(&[r, g, b, alpha]);
        }
    }
    out
}

/// How much of pixel (x, y) lies inside a rounded square of side `n` (0 to 1, 4×4 supersampled).
fn rounded_cover(n: u32, x: u32, y: u32) -> f32 {
    const S: u32 = 4;
    let side = n as f32;
    let r = side * CORNER;
    let half = side / 2.0;
    let mut inside = 0;
    for sy in 0..S {
        for sx in 0..S {
            let px = x as f32 + (sx as f32 + 0.5) / S as f32;
            let py = y as f32 + (sy as f32 + 0.5) / S as f32;
            // Distance past the straight edges, measured from the corner circles' centres.
            let qx = ((px - half).abs() - (half - r)).max(0.0);
            let qy = ((py - half).abs() - (half - r)).max(0.0);
            if qx * qx + qy * qy <= r * r {
                inside += 1;
            }
        }
    }
    inside as f32 / (S * S) as f32
}
