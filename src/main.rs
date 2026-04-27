#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)] // it's an example

use crate::egui::text::CCursorRange;
use crate::state::{Highlight, HlConfig, State, TextArea};
use eframe::egui;
use egui::FontData;
use egui::FontDefinitions;
use egui::FontFamily;
use egui::Galley;
use egui::text::CCursor;
use egui::{Rect, pos2};
use rfd;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub mod state;

fn get_text(cursor_range: CCursorRange, galley: &Arc<Galley>) -> String {
    let full_text = galley.text();

    // 2. 确定逻辑索引的起止点（确保顺序正确）
    let start_char_idx = cursor_range.primary.index.min(cursor_range.secondary.index);
    let end_char_idx = cursor_range.primary.index.max(cursor_range.secondary.index);

    // 3. 将字符索引（Char Index）转换为字节索引（Byte Index）
    // 因为 Rust 的 String 截取是基于字节的
    let byte_start = full_text
        .char_indices()
        .nth(start_char_idx)
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    let byte_end = full_text
        .char_indices()
        .nth(end_char_idx)
        .map(|(idx, _)| idx)
        .unwrap_or(full_text.len());

    // 4. 安全截取并返回
    full_text[byte_start..byte_end].to_string()
}

fn delete_selected_text(text: &mut String, range: egui::text::CCursorRange) {
    // 1. 确定字符索引范围
    let start_idx = range.primary.index.min(range.secondary.index);
    let end_idx = range.primary.index.max(range.secondary.index);

    // 2. 转换为字节偏移量（处理中文等宽字符的关键）
    let byte_start = text
        .char_indices()
        .nth(start_idx)
        .map(|(i, _)| i)
        .unwrap_or(text.len());
    let byte_end = text
        .char_indices()
        .nth(end_idx)
        .map(|(i, _)| i)
        .unwrap_or(text.len());

    // 3. 执行删除
    text.drain(byte_start..byte_end);
}

fn save(state: &State) {
    let json = serde_json::to_string_pretty(&state.text_area.highlights).unwrap();
    fs::write(&state.path.with_extension("json"), json).unwrap();
}

fn add_highlight(state: &mut State, config: HlConfig) {
    let start = state.text_area.selected_range.primary.index;
    let end = state.text_area.selected_range.secondary.index;
    state
        .text_area
        .highlights
        .entry(config)
        .or_insert_with(Vec::new)
        .push(Highlight::new(start, end));
}

// !TODO 为不同颜色的高亮进行循环mesh添加
fn galley_paint(fragements: Vec<CCursorRange>, own_galley: &mut Galley, config: HlConfig) {
    let (font_color, bg_color) = config.colors();
    for fragement in fragements {
        let [min, max] = fragement.sorted_cursors();
        let min = own_galley.layout_from_cursor(min);
        let max = own_galley.layout_from_cursor(max);

        for ri in min.row..=max.row {
            let placed_row = &mut own_galley.rows[ri];
            let row = Arc::make_mut(&mut placed_row.row);

            let left = if ri == min.row {
                row.x_offset(min.column)
            } else {
                0.0
            };
            let right = if ri == max.row {
                row.x_offset(max.column)
            } else {
                let newline_size = if placed_row.ends_with_newline {
                    row.height() / 2.0 // visualize that we select the newline
                } else {
                    0.0
                };
                row.size.x + newline_size
            };

            let rect = Rect::from_min_max(pos2(left, 0.0), pos2(right, row.size.y));
            let mesh = &mut row.visuals.mesh;

            if !row.glyphs.is_empty() {
                // Change color of the selected text:
                let first_glyph_index = if ri == min.row { min.column } else { 0 };
                let last_glyph_index = if ri == max.row {
                    max.column
                } else {
                    row.glyphs.len()
                };

                let first_vertex_index = row
                    .glyphs
                    .get(first_glyph_index)
                    .map_or(row.visuals.glyph_vertex_range.end, |g| g.first_vertex as _);
                let last_vertex_index = row
                    .glyphs
                    .get(last_glyph_index)
                    .map_or(row.visuals.glyph_vertex_range.end, |g| g.first_vertex as _);

                for vi in first_vertex_index..last_vertex_index {
                    mesh.vertices[vi].color = font_color //font_color
                }
            }

            // Time to insert the selection rectangle into the row mesh.
            // It should be on top (after) of any background in the galley,
            // but behind (before) any glyphs. The row visuals has this information:
            let glyph_index_start = row.visuals.glyph_index_start;

            // Start by appending the selection rectangle to end of the mesh, as two triangles (= 6 indices):
            let num_indices_before = mesh.indices.len();
            mesh.add_colored_rect(rect, bg_color); // background color
            assert_eq!(
                num_indices_before + 6,
                mesh.indices.len(),
                "We expect exactly 6 new indices"
            );

            // Copy out the new triangles:
            let selection_triangles = [
                mesh.indices[num_indices_before],
                mesh.indices[num_indices_before + 1],
                mesh.indices[num_indices_before + 2],
                mesh.indices[num_indices_before + 3],
                mesh.indices[num_indices_before + 4],
                mesh.indices[num_indices_before + 5],
            ];

            // Move every old triangle forwards by 6 indices to make room for the new triangle:
            for i in (glyph_index_start..num_indices_before).rev() {
                mesh.indices.swap(i, i + 6);
            }
            // Put the new triangle in place:
            mesh.indices[glyph_index_start..glyph_index_start + 6]
                .clone_from_slice(&selection_triangles);

            row.visuals.mesh_bounds = mesh.calc_bounds();
        }
    }
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 360.0]),
        ..Default::default()
    };

    let mut fonts = FontDefinitions::default();
    let font_bytes = std::fs::read("resources/NotoSerifCJK-Regular.ttc").expect("无法读取字体文件");
    fonts.font_data.insert(
        "my_font".to_owned(),
        FontData::from_owned(font_bytes).into(),
    );

    // 将该字体加入到字体族中 (Proportional 是比例字体，用于普通文本)
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "my_font".to_owned()); // 插入到第 0 位表示最高优先级

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            cc.egui_ctx.set_fonts(fonts);
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    pub state: State,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: State {
                path: PathBuf::from("text/a.txt"),
                content: "".to_owned(),
                changed: true,
                text_area: TextArea {
                    popup_pos: egui::Vec2::ZERO,
                    font_size: 20.0,
                    highlights: HashMap::from([(HlConfig::Blue, vec![])]),
                    selected_range: CCursorRange::two(CCursor::new(0), CCursor::new(0)),
                },
            },
        }
    }
}

impl eframe::App for MyApp {
    // shortcuts binding
    fn raw_input_hook(&mut self, _ctx: &egui::Context, _raw_input: &mut egui::RawInput) {
        _ctx.options_mut(|opt| {
            opt.zoom_with_keyboard = false;
        });

        _raw_input.events.retain(|event| {
            match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    if modifiers.ctrl {
                        if *key == egui::Key::S {
                            save(&self.state);
                            return false;
                        }

                        if (*key == egui::Key::Plus) || (*key == egui::Key::Equals) {
                            self.state.text_area.font_size += 1.0;
                        } else if *key == egui::Key::Minus {
                            self.state.text_area.font_size -= 1.0;
                        }
                        if !self.state.text_area.selected_range.is_empty() {
                            if *key == egui::Key::H {
                                add_highlight(&mut self.state, HlConfig::Blue);
                                return false;
                            }
                            if *key == egui::Key::Y {
                                add_highlight(&mut self.state, HlConfig::Yellow);
                                return false;
                            }
                        }
                    }

                    true // 如果不是我们要拦截的快捷键（比如普通打字、Ctrl+C），返回 true 保留它
                }
                _ => true, // 不是按键事件（比如鼠标移动），也返回 true 保留它
            }
        });
    }

    // the whole ui
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // navigator
        egui::Panel::top("top panel")
            .frame(egui::Frame::default().fill(egui::Color32::GRAY))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.state.changed {
                        self.state.content =
                            fs::read_to_string(&self.state.path).unwrap_or_default();

                        if fs::exists(&self.state.path.with_extension("json")).unwrap() {
                            let content =
                                fs::read_to_string(&self.state.path.with_extension("json"))
                                    .unwrap();
                            self.state.text_area.highlights =
                                serde_json::from_str(&content).unwrap();
                        } else {
                            fs::write(
                                &self.state.path.with_extension("json"),
                                serde_json::to_string_pretty(&self.state.text_area.highlights)
                                    .unwrap(),
                            )
                            .unwrap();
                        }
                        self.state.changed = false;
                    }

                    let save_button = egui::Button::new("Save").fill(egui::Color32::BLUE);

                    if ui.add(save_button).clicked() {
                        save(&self.state)
                    }
                    if ui.button("Change Path").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.state.path = path;
                            self.state.changed = true;
                        }
                    }
                });
            });

        // text area
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::GRAY))
            .show_inside(ui, |ui| {
                let text_edit = (egui::TextEdit::multiline(&mut self.state.content)
                    .desired_width(f32::INFINITY)
                    .desired_rows(20)
                    .background_color(egui::Color32::WHITE))
                .hint_text("Type something!")
                .layouter(&mut |ui, string, wrap_width| {
                    let layout_job = egui::text::LayoutJob::simple(
                        string.as_str().to_string(),
                        egui::FontId::monospace(self.state.text_area.font_size),
                        egui::Color32::BLACK,
                        wrap_width,
                    );

                    let galley = ui.fonts_mut(|f| f.layout_job(layout_job));
                    let mut own_galley = (*galley).clone();

                    //load highlight
                    for (config, highlights) in &self.state.text_area.highlights {
                        let mut fragements = vec![];
                        for highlight in highlights {
                            fragements.push(egui::text::CCursorRange::two(
                                egui::text::CCursor::new(highlight.start),
                                egui::text::CCursor::new(highlight.end),
                            ));
                        }
                        galley_paint(fragements, &mut own_galley, *config);
                    }

                    Arc::new(own_galley)
                })
                .show(ui);
                let galley = text_edit.galley;

                if let Some(cursor_range) = text_edit.cursor_range {
                    if !cursor_range.is_empty() {
                        self.state.text_area.selected_range = cursor_range;
                        let rect = galley.pos_from_cursor(cursor_range.primary);
                        let screen_pos = rect.right_top().to_vec2();
                        self.state.text_area.popup_pos = screen_pos;

                        egui::Area::new(egui::Id::new("my_area"))
                            .fixed_pos(
                                (self.state.text_area.popup_pos + egui::vec2(-10.0, -60.0))
                                    .to_pos2(),
                            )
                            .order(egui::Order::Foreground) // 确保在最前面
                            .show(ui, |ui| {
                                egui::Frame::popup(ui.style()).show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.button("Copy").clicked() {
                                            // 4. 安全截取并返回
                                            let text = get_text(cursor_range, &galley);
                                            let mut clipboard =
                                                arboard::Clipboard::new().expect("");
                                            match clipboard.set_text(text) {
                                                Ok(_) => println!("成功复制到剪贴板！"),
                                                Err(e) => eprintln!("复制失败: {}", e),
                                            };
                                        }
                                        if ui.button("Cut").clicked() {
                                            let text = get_text(cursor_range, &galley);
                                            let mut clipboard =
                                                arboard::Clipboard::new().expect("");
                                            match clipboard.set_text(text) {
                                                Ok(_) => println!("成功复制到剪贴板！"),
                                                Err(e) => eprintln!("复制失败: {}", e),
                                            };
                                            delete_selected_text(
                                                &mut self.state.content,
                                                cursor_range,
                                            );
                                        }
                                        if ui.button("Pastle").clicked() {}
                                        if ui.button("Highlight").clicked() {
                                            add_highlight(&mut self.state, HlConfig::Blue);
                                        }
                                        if ui.button("Mark").clicked() {}
                                    });
                                });
                            });
                    }
                }
            });
    }
}
