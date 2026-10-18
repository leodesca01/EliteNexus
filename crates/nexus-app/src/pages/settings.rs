//! Settings, laid out like System Settings: grouped rows with the control on the right.

use crate::core::{CoreStatus, Phase};
use crate::settings::Settings;
use crate::theme::{pal, size};
use crate::widgets::{self, cards, footnote, group, section};
use egui::{RichText, Ui};

pub struct SettingsCtx<'a> {
    pub settings: &'a mut Settings,
    pub status: &'a CoreStatus,
    pub data_dir: &'a std::path::Path,
    pub changed: bool,
    pub test_voice: bool,
    pub restart_core: bool,
}

/// A slider with its value shown on the right, sized for a settings row.
fn slider(ui: &mut Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>, decimals: usize, suffix: &str) -> bool {
    ui.add_sized([64.0, 20.0], egui::Label::new(RichText::new(format!("{:.*}{suffix}", decimals, *value)).color(pal::DIM)));
    ui.add(egui::Slider::new(value, range).show_value(false)).changed()
}

const UI_SCALES: [(f32, &str); 6] = [(0.9, "90 %"), (1.0, "100 %"), (1.1, "110 %"), (1.25, "125 %"), (1.5, "150 %"), (1.75, "175 %")];

pub fn show(ui: &mut Ui, cx: &mut SettingsCtx<'_>) {
    cards(ui, 2, 420.0, |c| {
        c.add(|ui| {
            let s = &mut *cx.settings;
            section(ui, "Voice and alerts");
            group(ui, |g| {
                cx.changed |= g.switch(&mut s.tts_enabled, "Speak alerts", "Text-to-speech for jumps, scans, missions and price alerts.");
                g.row("Speak from priority", "", |ui| {
                    egui::ComboBox::from_id_salt("ttsprio").selected_text(["Low", "Normal", "High", "Critical"][s.tts_min_priority.min(3) as usize]).show_ui(ui, |ui| {
                        for (i, n) in ["Low", "Normal", "High", "Critical"].iter().enumerate() {
                            cx.changed |= ui.selectable_value(&mut s.tts_min_priority, i as u8, *n).changed();
                        }
                    });
                });
                g.row("Speech rate", "", |ui| cx.changed |= slider(ui, &mut s.tts_rate, 0.5..=2.0, 2, "×"));
                g.row("Volume", "", |ui| {
                    let mut pct = s.tts_volume * 100.0;
                    if slider(ui, &mut pct, 0.0..=100.0, 0, " %") {
                        s.tts_volume = pct / 100.0;
                        cx.changed = true;
                    }
                });
                g.row("Banners stay for", "", |ui| cx.changed |= slider(ui, &mut s.toast_seconds, 2.0..=30.0, 0, " s"));
                g.content(|ui| {
                    if widgets::button(ui, "Test voice").clicked() {
                        cx.test_voice = true;
                    }
                });
            });
            section(ui, "Exploration");
            group(ui, |g| {
                g.row("High-value body", "Bodies worth at least this much are highlighted and announced.", |ui| {
                    let mut hv = s.high_value_threshold as f32 / 1e6;
                    if slider(ui, &mut hv, 0.1..=10.0, 1, " M Cr") {
                        s.high_value_threshold = (hv * 1e6) as i64;
                        cx.changed = true;
                    }
                });
            });
            section(ui, "Display");
            group(ui, |g| {
                // Fixed steps, not a slider: zooming while dragging rescales the slider under the
                // pointer, which runs the value off to either end.
                g.content(|ui| {
                    ui.label(RichText::new("Text and controls size").color(pal::TEXT));
                    let mut scale = s.ui_scale;
                    if widgets::segmented(ui, &mut scale, &UI_SCALES) {
                        s.ui_scale = scale;
                        cx.changed = true;
                    }
                });
            });
            footnote(ui, "Ctrl + and Ctrl − also change the size; Ctrl 0 goes back to 100 %.");
        });
        c.add(|ui| {
            let st = cx.status;
            section(ui, "Journal");
            group(ui, |g| {
                let phase = match &st.phase {
                    Phase::Starting => "Starting".to_string(),
                    Phase::Loading { done, total } => format!("Loading {done} of {total}"),
                    Phase::Live => "Live".to_string(),
                    Phase::Error(e) => format!("Error: {e}"),
                };
                g.value("State", phase);
                g.row("Folder", "", |ui| {
                    let folder = st.journal_dir.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "Not found".into());
                    ui.add(egui::Label::new(RichText::new(folder.clone()).color(pal::DIM)).truncate()).on_hover_text(folder);
                });
                g.value("Journal files", format!("{} · {:.1} MB", st.files, st.bytes as f64 / 1e6));
                g.value("History load time", format!("{:.0} ms", st.load_ms));
                g.row("Following", "", |ui| {
                    ui.add(egui::Label::new(RichText::new(st.current_file.clone().unwrap_or_default()).color(pal::DIM)).truncate());
                });
                g.value("Live events this run", st.live_events.to_string());
                g.content(|ui| {
                    ui.label(RichText::new("Custom journal folder").size(size::FOOTNOTE).color(pal::DIM));
                    let mut text = cx.settings.journal_dir.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
                    ui.horizontal(|ui| {
                        let w = (ui.available_width() - 150.0).max(160.0);
                        if ui.add(egui::TextEdit::singleline(&mut text).hint_text("Empty: found automatically").desired_width(w).margin(egui::Margin::symmetric(10, 6))).changed() {
                            cx.settings.journal_dir = if text.trim().is_empty() { None } else { Some(text.trim().into()) };
                            cx.changed = true;
                        }
                        if widgets::button(ui, "Reload journals").clicked() {
                            cx.restart_core = true;
                        }
                    });
                    if cfg!(not(windows)) {
                        let hint = if cfg!(target_os = "macos") {
                            "macOS: Elite Nexus looks in the game's prefix in every Steam library, where NotProton (Steam Play for macOS) keeps it (~/Library/Application Support/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous), then in CrossOver's bottles, ~/.wine and Whisky's bottles. For any other Wine prefix, enter its folder here: Bindings and HUD colours then use the Options folder in that same prefix."
                        } else {
                            "Linux: Elite Nexus looks in the game's Proton prefix in every Steam library (steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous), then in Lutris's and Heroic's usual folders. For any other Wine prefix, enter its folder here: Bindings and HUD colours then use the Options folder in that same prefix."
                        };
                        ui.add(egui::Label::new(RichText::new(hint).size(size::FOOTNOTE).color(pal::DIM)).wrap());
                    }
                });
                if !st.errors.is_empty() {
                    g.content(|ui| {
                        for e in st.errors.iter().rev().take(5) {
                            ui.add(egui::Label::new(RichText::new(e).size(size::FOOTNOTE).color(pal::BAD)).wrap());
                        }
                    });
                }
            });
            section(ui, "About");
            group(ui, |g| {
                g.value("Version", format!("Elite Nexus {}", env!("CARGO_PKG_VERSION")));
                g.row("Data folder", "", |ui| {
                    let d = cx.data_dir.display().to_string();
                    ui.add(egui::Label::new(RichText::new(d.clone()).color(pal::DIM)).truncate()).on_hover_text(d);
                });
                #[cfg(target_os = "linux")]
                app_menu(g);
            });
            footnote(ui, "Elite Nexus is free software under the GNU GPL, version 3 or later, and comes with no warranty. Source: github.com/uruskan/EliteNexus. Elite Dangerous is a trademark of Frontier Developments plc; Elite Nexus is an unofficial community tool.");
        });
    });
}

/// Linux: Elite Nexus in the desktop's app menu (desktop_entry.rs), written only on a click here.
#[cfg(target_os = "linux")]
fn app_menu(g: &mut widgets::Group) {
    use crate::desktop_entry::{self, State};
    let (Some(home), Ok(exe)) = (crate::settings::user_data_base(), std::env::current_exe()) else { return };
    let (state_id, msg_id) = (egui::Id::new("settings-app-menu"), egui::Id::new("settings-app-menu-msg"));
    let ctx = g.ui.ctx().clone();
    let state = match ctx.data(|d| d.get_temp::<State>(state_id)) {
        Some(s) => s,
        None => {
            let s = desktop_entry::state(&home, &exe);
            ctx.data_mut(|d| d.insert_temp(state_id, s.clone()));
            s
        }
    };
    let (detail, action) = match &state {
        State::Missing => (format!("Adds Elite Nexus and its icon to your desktop's app menu ({}).", desktop_entry::entry_path(&home).display()), Some("Add")),
        State::Current => ("Elite Nexus is in your desktop's app menu, with its icon.".to_string(), None),
        State::Elsewhere(p) => (format!("Your app menu starts {p}, not this copy."), Some("Point it here")),
    };
    // Some(true): add or update; Some(false): remove.
    let clicked = g.row("App menu", &detail, |ui| {
        let mut act = None;
        if let Some(label) = action
            && widgets::button(ui, label).clicked()
        {
            act = Some(true);
        }
        if state != State::Missing && widgets::button(ui, "Remove").clicked() {
            act = Some(false);
        }
        act
    });
    if let Some(add) = clicked {
        let done = if add { desktop_entry::install(&home, &exe) } else { desktop_entry::uninstall(&home) };
        let msg = done.err().map(|e| format!("Couldn't change the app menu: {e}"));
        ctx.data_mut(|d| {
            d.insert_temp(state_id, desktop_entry::state(&home, &exe));
            d.insert_temp(msg_id, msg);
        });
    }
    if let Some(Some(msg)) = ctx.data(|d| d.get_temp::<Option<String>>(msg_id)) {
        g.content(|ui| ui.add(egui::Label::new(RichText::new(msg).size(size::FOOTNOTE).color(pal::BAD)).wrap()));
    }
}
