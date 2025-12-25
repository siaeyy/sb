use std::sync::Arc;

use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    text::Span,
    widgets::{Block, BorderType, Padding, Widget},
    crossterm::style::Color,
    style::{Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Borders, Paragraph, StatefulWidget, Wrap},
};

use tui_input::Input;

use crate::{
    descriptions::Description,
    states::{BinaryListState, CursorState}
};

pub struct SearchInput<'inner, 'cursor> {
    pub inner: &'inner Input,
    pub cursor_state: &'cursor mut CursorState,
}

impl<'a, 'b> SearchInput<'a, 'b> {
    fn get_cursor_position(input: &Input, area: Rect) -> Position {
        let width = area.width.max(3) - 4;
        let scroll = input.visual_scroll(width as usize);

        let x = input.visual_cursor().max(scroll) - scroll + 2;

        (area.x + x as u16, area.y + 1).into()
    }
}

impl<'a, 'b> Widget for SearchInput<'a, 'b> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = self.inner.value();
        let text_span = Span::raw(text);

        let outer_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::left(1));

        let inner_area = outer_block.inner(area);

        outer_block.render(area, buf);
        text_span.render(inner_area, buf);

        let cursor_pos = Self::get_cursor_position(&self.inner, area);
        self.cursor_state.position = Some(cursor_pos).into();
    }
}

pub struct SearchResultItem<'bin> {
    pub name: &'bin String,
    pub description: Option<Arc<Description>>,
}

#[derive(PartialEq, Eq)]
pub enum SearchResultItemOrder {
    Selected,
    Last,
    Rest,
}

impl<'a> SearchResultItem<'a> {
    pub fn calculate_height(&self, area: &Rect) -> u16 {
        let calculate_desc_height = |desc: &Description| {
            let mut height = 0;

            for line in desc.value.lines() {
                let readable_len = line.chars().count();
                let over = readable_len as u16 / area.width;

                height += 1 + over;
            }

            height
        };

        let description_height = self
            .description
            .as_ref()
            .map(|d| calculate_desc_height(d.as_ref()) + 1)
            .unwrap_or(0);

        return description_height + 2;
    }
}

impl<'a> StatefulWidget for SearchResultItem<'a> {
    type State = SearchResultItemOrder;

    fn render(self, area: Rect, buf: &mut Buffer, order: &mut Self::State) {
        let bg = match order {
            Self::State::Selected => Color::DarkCyan,
            _ => Color::Reset,
        };

        let borders = match order {
            Self::State::Last => Borders::NONE,
            _ => Borders::BOTTOM,
        };

        let block = Block::new()
            .borders(borders)
            .padding(Padding::horizontal(1))
            .border_style(Style::new().dark_gray());

        let title = Line::styled(
            self.name,
            Style::default()
                .white()
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );

        let mut text = Text::from(title);

        if let Some(desc) = &self.description {
            let description = Text::from("\n".to_owned() + desc.value.as_str());
            text.extend(description);
        }

        let item = Paragraph::new(text).bg(bg).wrap(Wrap { trim: true });
        let mut item_area = area.clone();

        item_area.height -= 1;

        item.render(item_area, buf);
        block.render(area, buf);
    }
}

pub struct SearchResultList<'bins> {
    pub binary_list: &'bins BinaryListState,
}

impl<'a> Widget for SearchResultList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let selected = self.binary_list.selected;
        let binaries = (&self.binary_list.binaries)
            .ordered_iter()
            .skip(selected as usize);

        let max_y = area.y + area.height;
        let mut height_offset = 0;

        for (i, binary) in binaries.enumerate() {
            let readable_binary = &binary.read().unwrap();

            let name = &readable_binary.name;
            let description = readable_binary.get_description();
            
            let item = SearchResultItem { name, description };

            let item_height = item.calculate_height(&area);
            let mut item_area = area.clone();

            item_area.y += height_offset;
            item_area.height = item_height.min(max_y - item_area.y);
            height_offset += item_area.height;

            let is_last = height_offset > area.height - 1;

            let mut order = if i == selected.to_owned() as usize {
                SearchResultItemOrder::Selected
            } else if is_last {
                SearchResultItemOrder::Last
            } else {
                SearchResultItemOrder::Rest
            };

            item.render(item_area, buf, &mut order);

            if is_last {
                break;
            }
        }
    }
}

pub struct SearchResult<'bins> {
    pub binary_list: Option<&'bins BinaryListState>,
}

impl<'a> Widget for SearchResult<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let result_block = Block::bordered().border_type(BorderType::Rounded);

        (&result_block).render(area, buf);

        let binary_list = match self.binary_list {
            Some(v) => v,
            _ => return,
        };

        let list_area = result_block.inner(area);
        let list = SearchResultList { binary_list };

        list.render(list_area, buf);
    }
}
