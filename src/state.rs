use crate::egui::text::CCursorRange;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub enum Hl_config {
    Blue,
    Yellow,
}

impl Hl_config {
    pub fn colors(&self) -> (egui::Color32, egui::Color32) {
        match self {
            Hl_config::Blue => (egui::Color32::WHITE, egui::Color32::BLUE),
            Hl_config::Yellow => (egui::Color32::WHITE, egui::Color32::YELLOW),
        }
    }
}

pub struct TextArea {
    pub popup_pos: egui::Vec2,
    pub font_size: f32,
    pub highlights: Vec<Highlight>,
    pub selected_range: CCursorRange,
}

#[derive(Serialize, Deserialize)]
pub struct Highlight {
    pub start: usize,
    pub end: usize,
}

impl Highlight {
    pub fn new(start: usize, end: usize) -> Self {
        if start > end {
            Self {
                start: end,
                end: start,
            }
        } else {
            Self {
                start: start,
                end: end,
            }
        }
    }
}

pub struct State {
    pub path: PathBuf,
    pub content: String,
    pub changed: bool,
    pub text_area: TextArea,
}
