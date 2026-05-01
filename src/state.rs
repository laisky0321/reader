use crate::egui::text::CCursorRange;
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

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Highlight {
    pub start: usize,
    pub end: usize,
    pub id: usize,
}

impl Highlight {
    pub fn new(start: usize, end: usize, id: usize) -> Self {
        if start > end {
            Self {
                start: end,
                end: start,
                id: id,
            }
        } else {
            Self {
                start: start,
                end: end,
                id: id,
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BookMark {
    pub id: usize,
    pub index: usize,
    pub content: String,
    pub y: f32,
}

impl BookMark {
    pub fn new(index: usize, content: String, id: usize) -> Self {
        Self {
            id: id,
            index: index,
            content: content,
            y: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Image {
    pub id: usize,
    pub index: usize,
    pub path: PathBuf,
    pub y: f32,
}

impl Image {
    pub fn new(index: usize, path: PathBuf, id: usize) -> Self {
        Self {
            id: id,
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct TextChange {
    pub change_index: usize,
    pub change_range: i32,
    pub index: usize,
}

pub struct StringWithAnnotation<'a> {
    pub text: &'a mut String,
    pub changes: TextChange,
}

impl<'a> StringWithAnnotation<'a> {
    fn highlights(&self) -> Option<&HashMap<HlConfig, Vec<Highlight>>> {
        None
    }
}

pub struct State {
    pub path: PathBuf,
    pub content: String,
    pub changed: bool,
    pub text_area: TextArea,
    pub annotation: Annotation,
    pub text_change: TextChange,
    pub focus_id: usize,
}
