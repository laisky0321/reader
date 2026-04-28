use crate::egui::text::CCursorRange;
use eframe::wgpu::ContextBlasBuildEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

#[derive(Serialize, Deserialize)]
pub struct BookMark {
    pub index: usize,
    content: String,
    pub y: f32,
}

impl BookMark {
    pub fn new(index: usize, content: String) -> Self {
        Self {
            index: index,
            content: content,
            y: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Image {
    pub index: usize,
    pub path: PathBuf,
    pub y: f32,
}

impl Image {
    pub fn new(index: usize, path: PathBuf) -> Self {
        Self {
            index: index,
            path: path,
            y: 0.0,
        }
    }
    pub fn set_y(self: &mut Self, y: f32) {
        self.y = y;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Annotation {
    pub bookmarks: Vec<BookMark>,
    pub images: Vec<Image>,
    pub highlights: HashMap<HlConfig, Vec<Highlight>>,
}

pub struct State {
    pub path: PathBuf,
    pub content: String,
    pub changed: bool,
    pub text_area: TextArea,
    pub annotation: Annotation,
}
