use std::{
    sync::{
        Arc,
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use color_eyre::eyre::Result as RepResult;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEvent, poll},
    layout::{Constraint, Direction, Layout},
};

use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    app::App, binaries::{
        BinaryNode,
        attach_manpaths,
        search_binaries,
    },
    cli::Cli,
    states::*,
    widgets::*
};

const APP_ROOT_LAYOUT: LazyLock<Layout> = LazyLock::new(|| -> Layout {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Max(80),
            Constraint::Min(0),
        ])
});

const APP_LAYOUT: LazyLock<Layout> = LazyLock::new(|| -> Layout {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
});

pub struct InteractiveApp {
    args: Option<Cli>,

    is_running: bool,
    redraw: Arc<AtomicBool>,

    input: Input,
    result: Option<BinaryListState>,
    
    #[cfg(debug_assertions)]
    tick_state: TickState,
    cursor_state: CursorState,
}

impl Default for InteractiveApp {
    fn default() -> Self {
        let redraw = Arc::new(AtomicBool::new(false));
        let input = Input::default().with_cursor(1).into();

        #[cfg(debug_assertions)]
        let tick_state = TickState::default();
        let cursor_state = CursorState::default();

        Self {
            args: None,
            is_running: true,
            redraw,
            input,
            result: None,
            #[cfg(debug_assertions)]
            tick_state,
            cursor_state,
        }
    }
}

impl App for InteractiveApp {
    fn with_args(args: Cli) -> Self {
        let input_arg = args.input.clone();

        let mut self_ = Self::default();
        self_.args = Some(args);

        if let Some(v) = input_arg {
            let input = self_.input.with_value(v);
            self_.input = input;
            self_.handle_post_input();
        }

        self_
    }

    fn run(&mut self) -> RepResult<()> {
        let terminal = ratatui::init();
        self.run_tui(terminal)
            .map(|_| {
                ratatui::restore();
            })
    }
}

impl InteractiveApp {
    #[cfg(debug_assertions)]
    fn count_tick(&mut self) {
        let tick = &mut self.tick_state;

        if tick.start.elapsed().as_secs() > 1 {
            tick.rate = tick.count;
            tick.count = 0;
            tick.start = Instant::now();
        } else {
            tick.count += 1;
        }
    }

    #[cfg(debug_assertions)]
    fn debug_tick(&self, frame: &mut Frame) {
        use ratatui::text::Span;

        let tps_panel_text = format!(
            "Last TPS: {}",
            self.tick_state.rate.to_string()
        );

        let tps_panel = Span::raw(tps_panel_text);

        frame.render_widget(tps_panel, frame.area());
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [_, root_area, _] = APP_ROOT_LAYOUT.areas(frame.area());

        let [
            result_area,
            input_area,
        ] = APP_LAYOUT.areas(root_area);

        let search_input = SearchInput {
            inner: &self.input,
            cursor_state: &mut self.cursor_state,
        };

        let search_result = SearchResult {
            binary_list: self.result.as_ref(),
        };

        frame.render_widget(search_input, input_area);
        frame.render_widget(search_result, result_area);

        if let Some(cursor_pos) = self.cursor_state.position {
            frame.set_cursor_position(cursor_pos);
        }

        #[cfg(debug_assertions)]
        self.debug_tick(frame);
    }

    fn run_tui(&mut self, mut terminal: DefaultTerminal) -> RepResult<()> {
        while self.is_running {
            #[cfg(debug_assertions)]
            self.count_tick();

            terminal.draw(|frame| self.draw(frame))?;

            if self.wait_event()? {
                self.event_handler(event::read()?);
            }
        }

        Ok(())
    }

    fn exit(&mut self) {
        self.is_running = false;
    }

    const HEAT_RANGE: usize = 8;

    fn get_hot_binaries<'s>(&'s self) -> Vec<BinaryNode> {
        let (
            binaries,
            selected,
        ) = match &self.result {
            Some(v) => (
                &v.binaries,
                v.selected,
            ),
            None => return vec![],
        };

        let range: usize = Self::HEAT_RANGE;

        let start_ = selected as isize - range as isize;
        let start_ = if start_ <= 0 { start_ } else { 0 } as usize;

        let start = selected.min(start_);
        let count = (selected + range) - start;

        let hot_binaries_iter = binaries
            .values()
            .rev()
            .skip(start)
            .take(count);

        hot_binaries_iter.fold(
            Vec::with_capacity(count),
            |mut acc, v| {
                acc.push(v.clone());
                acc
            },
        )
    }

    fn search(&mut self) {
        let value = self.input.value();

        if value.is_empty() {
            self.result = None;
            return;
        }

        let result = BinaryListState {
            binaries: search_binaries(value),
            selected: 0,
        };

        self.result = Some(result);
    }

    fn add_descriptions(&self) {
        let redraw_req = self.redraw.clone();
        let hot_binaries = self.get_hot_binaries();

        rayon::spawn(move || {
            attach_manpaths(&hot_binaries);
            redraw_req.store(true, Ordering::Release);
        });
    }

    fn handle_post_input(&mut self) {
        self.search();
    
        let args = match &self.args {
            Some(v) => v,
            None => return,
        };

        if args.show_descriptions {
            self.add_descriptions();
        }
    }

    fn key_event_handler(&mut self, event: KeyEvent) {
        match event.code {
            event::KeyCode::Esc => self.exit(),
            event::KeyCode::Up => {},
            event::KeyCode::Down => {},
            _ => {},
        }
    }

    fn event_handler(&mut self, event: Event) {
        if let Some(_) = self.input.handle_event(&event) {
            self.handle_post_input();
            return;
        }

        match event {
            Event::Key(e) => self.key_event_handler(e),
            _ => {}
        }
    }

    fn wait_event(&mut self) -> RepResult<bool> {
        loop {
            let redraw_req = self.redraw
                .swap(false, Ordering::AcqRel);

            if !self.is_running || redraw_req {
                return Ok(false);
            }

            if poll(Duration::from_millis(100))? {
                return Ok(true);
            }
        }
    }
}
