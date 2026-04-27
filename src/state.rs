use crate::egui::text::CCursorRange;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum HlConfig {
    Blue,
    Yellow,
}

impl HlConfig {
    pub fn colors(&self) -> (egui::Color32, egui::Color32) {
        match self {
            HlConfig::Blue => (egui::Color32::BLACK, egui::Color32::BLUE),
            HlConfig::Yellow => (egui::Color32::BLACK, egui::Color32::YELLOW),
        }
    }
}

pub struct TextArea {
    pub popup_pos: egui::Vec2,
    pub font_size: f32,
    pub highlights: HashMap<HlConfig, Vec<Highlight>>,
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
