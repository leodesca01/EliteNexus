//! Key bindings: active presets, conflicts, missing essentials, searchable action list, and a
//! printable reference card (EDRefCard replacement).

use crate::theme::{display, pal};
use crate::widgets::{badge, card, empty};
use egui::{RichText, Ui};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular as icon;
use nexus_binds::{analyze, find_presets, render_html, Bindings, Context, Importance, PresetCategory, PresetDirectory, Report, Slot};

pub struct Loaded {
    pub dir: PresetDirectory,
    pub bindings: Bindings,
    pub report: Report,
}

#[derive(Default)]
pub struct BindsUi {
    pub loaded: Option<Result<Loaded, String>>,
    pub filter: String,
    pub context: Option<Context>,
    pub only_problems: bool,
    pub last_export: Option<String>,
}

impl BindsUi {
    pub fn load(&mut self) {
        let result = (|| -> Result<Loaded, String> {
            let dir_path = nexus_binds::default_bindings_dir().ok_or("The game's Options folder wasn't found (on Linux or macOS: set the journal folder in Settings to the one in the game's Wine prefix)")?;
            let dir = find_presets(&dir_path).map_err(|e| e.to_string())?;
            let builtin = builtin_dirs();
            let builtin_refs: Vec<&std::path::Path> = builtin.iter().map(|p| p.as_path()).collect();
            let bindings = match dir.load_effective(&builtin_refs) {
                Ok(b) => b,
                Err(_) => dir.primary().ok_or("No .binds presets found")?.load().map_err(|e| e.to_string())?,
            };
            let report = analyze(&bindings);
            Ok(Loaded { dir, bindings, report })
        })();
        self.loaded = Some(result);
    }
}

/// The game's ControlSchemes folders (built-in presets), if we can find the install.
fn builtin_dirs() -> Vec<std::path::PathBuf> {
    nexus_hud::install::find_installs().into_iter().map(|i| i.folder.join("ControlSchemes")).filter(|cs| cs.is_dir()).collect()
}

pub fn show(ui: &mut Ui, st: &mut BindsUi, data_dir: &std::path::Path) {
    if st.loaded.is_none() {
        st.load();
    }
    ui.horizontal(|ui| {
        if ui.button(format!("{} Reload", icon::ARROW_CLOCKWISE)).clicked() {
            st.load();
        }
        if let Some(Ok(l)) = &st.loaded
            && ui.button(format!("{} Printable reference card", icon::PRINTER)).on_hover_text("Writes an HTML card and opens it in your browser").clicked() {
                let html = render_html(&l.bindings, &l.report);
                let path = data_dir.join("bindings-refcard.html");
                match std::fs::write(&path, html) {
                    Ok(()) => {
                        // file:///C:/... on Windows, file:///home/... on Linux (whose paths already start with /).
                        let url = format!("file:///{}", path.display().to_string().replace('\\', "/").trim_start_matches('/'));
                        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                        st.last_export = Some(path.display().to_string());
                    }
                    Err(e) => st.last_export = Some(format!("Could not write card: {e}")),
                }
            }
        if let Some(p) = &st.last_export {
            ui.label(RichText::new(p).small().color(pal::DIM));
        }
    });
    ui.add_space(6.0);
    let loaded = match &st.loaded {
        Some(Ok(l)) => l,
        Some(Err(e)) => {
            empty(ui, icon::GAME_CONTROLLER, "Couldn't read your bindings", e);
            return;
        }
        None => return,
    };
    let r = &loaded.report;

    crate::widgets::cards(ui, 3, 280.0, |c| {
        c.add(|ui| {
            card(ui, "Active presets", |ui| {
            for cat in [PresetCategory::General, PresetCategory::Ship, PresetCategory::Srv, PresetCategory::OnFoot] {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(cat.label()).color(pal::DIM));
                    ui.label(RichText::new(loaded.dir.active_name(cat).unwrap_or("?")).color(pal::TEXT));
                });
            }
            ui.label(RichText::new(format!("{} actions bound · {} bindings", r.totals.bound_actions, r.totals.bindings)).small().color(pal::DIM));
        });
        });
        c.add(|ui| {
            card(ui, "Health", |ui| {
            let essentials = r.missing_essentials().count();
            ui.horizontal(|ui| {
                crate::widgets::stat(ui, "conflicts", r.conflicts.len().to_string(), if r.conflicts.is_empty() { pal::GOOD } else { pal::BAD });
                ui.add_space(14.0);
                crate::widgets::stat(ui, "missing essentials", essentials.to_string(), if essentials == 0 { pal::GOOD } else { pal::WARN });
                ui.add_space(14.0);
                crate::widgets::stat(ui, "recommended", (r.missing.len() - essentials).to_string(), pal::DIM);
            });
        });
        });
        c.add(|ui| {
            card(ui, "Devices", |ui| {
            ui.horizontal_wrapped(|ui| {
                for d in &r.devices {
                    let col = if d.conflicts > 0 { pal::WARN } else { pal::CYAN };
                    badge(ui, &format!("{} · {} btn · {} axes", d.name, d.buttons, d.axes), col);
                }
            });
        });
        });
    });
    ui.add_space(8.0);
    if !r.conflicts.is_empty() || r.missing.iter().any(|m| m.importance == Importance::Essential) {
        crate::widgets::cards(ui, 2, 360.0, |c| {
            c.add(|ui| {
                card(ui, "Conflicts", |ui| {
                if r.conflicts.is_empty() {
                    ui.label(RichText::new("None — nice.").color(pal::GOOD));
                }
                for cf in &r.conflicts {
                    ui.horizontal_wrapped(|ui| {
                        badge(ui, &cf.input.display(), pal::BAD);
                        ui.label(RichText::new(cf.actions.iter().map(|a| format!("{} ({})", a.name, a.context.label())).collect::<Vec<_>>().join("  ·  ")).color(pal::TEXT));
                    });
                }
            });
            });
            c.add(|ui| {
                card(ui, "Missing essentials", |ui| {
                let mut any = false;
                for m in r.missing_essentials() {
                    any = true;
                    ui.horizontal_wrapped(|ui| {
                        badge(ui, m.context.label(), pal::WARN);
                        ui.label(RichText::new(m.label).color(pal::TEXT));
                    });
                }
                if !any {
                    ui.label(RichText::new("All essentials bound.").color(pal::GOOD));
                }
            });
            });
        });
        ui.add_space(8.0);
    }

    ui.horizontal(|ui| {
        crate::widgets::search_field(ui, &mut st.filter, "Search actions or keys", 280.0);
        egui::ComboBox::from_id_salt("bindctx")
            .selected_text(st.context.map(|c| c.label()).unwrap_or("All contexts"))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut st.context, None, "All contexts");
                for c in [Context::Ship, Context::Srv, Context::OnFoot, Context::Interface, Context::GalaxyMap, Context::Scanners, Context::Fighter, Context::Multicrew, Context::Camera, Context::HoloMe, Context::General, Context::Colonisation, Context::Store] {
                    ui.selectable_value(&mut st.context, Some(c), c.label());
                }
            });
        crate::widgets::toggle(ui, &mut st.only_problems, "Only conflicts");
    });
    let f = st.filter.to_ascii_lowercase();
    let rows: Vec<_> = loaded
        .bindings
        .actions
        .iter()
        .filter(|a| a.is_bound())
        .map(|a| (a, a.info()))
        .filter(|(_, i)| !i.legacy)
        .filter(|(_, i)| st.context.is_none_or(|c| i.context == c))
        .filter(|(a, _)| !st.only_problems || r.is_conflicted(&a.action, Slot::Primary) || r.is_conflicted(&a.action, Slot::Secondary))
        .filter(|(a, i)| {
            f.is_empty()
                || i.name.to_ascii_lowercase().contains(&f)
                || a.inputs().any(|(_, inp)| inp.display().to_ascii_lowercase().contains(&f))
        })
        .collect();
    TableBuilder::new(ui)
        .striped(true)
        .resizable(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::initial(110.0))
        .column(Column::initial(170.0))
        .column(Column::initial(260.0))
        .column(Column::initial(200.0))
        .column(Column::initial(200.0))
        .column(Column::remainder())
        .header(30.0, |mut h| {
            for t in ["Context", "Category", "Action", "Primary", "Secondary", "Axis"] {
                h.col(|ui| {
                    crate::widgets::th(ui, t);
                });
            }
        })
        .body(|body| {
            body.rows(32.0, rows.len(), |mut row| {
                let (a, info) = &rows[row.index()];
                row.col(|ui| {
                    ui.label(RichText::new(info.context.label()).color(pal::DIM));
                });
                row.col(|ui| {
                    ui.label(RichText::new(info.category.label()).color(pal::DIM));
                });
                row.col(|ui| {
                    ui.label(RichText::new(info.name.as_ref()).color(pal::TEXT));
                });
                for slot in [Slot::Primary, Slot::Secondary] {
                    row.col(|ui| {
                        if let Some(inp) = a.input(slot) {
                            let bad = r.is_conflicted(&a.action, slot);
                            ui.label(RichText::new(inp.display()).font(display(13.0)).color(if bad { pal::BAD } else { pal::ACCENT_SOFT }));
                        }
                    });
                }
                row.col(|ui| {
                    if let Some(ax) = &a.axis {
                        ui.label(RichText::new(format!("{}{}", ax.input.display(), if ax.inverted { " (inverted)" } else { "" })).color(pal::CYAN));
                    }
                });
            });
        });
}
