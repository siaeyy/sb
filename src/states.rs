#[cfg(debug_assertions)]
use std::time::Instant;

use ratatui::layout::Position;

use crate::binaries::BinSearchResult;

#[derive(Default)]
pub struct CursorState {
    pub position: Option<Position>,
}

#[cfg(debug_assertions)]
pub struct TickState {
    pub count: u32,
    pub rate: u32,    
    pub start: Instant,
}

#[cfg(debug_assertions)]
impl Default for TickState {
    fn default() -> Self {
        Self {
            count: 0,
            rate: 0,
            start: Instant::now(),
        }
    }
}

pub struct BinaryListState {
    pub binaries: BinSearchResult,
    pub selected: usize,
}
