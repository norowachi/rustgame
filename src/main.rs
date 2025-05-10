use std::vec;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize, palette::tailwind},
    text::Text,
    widgets::{Cell, HighlightSpacing, Paragraph, Row, Table, TableState, Wrap},
};

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

struct TableColors {
    buffer_bg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    selected_column_style_fg: Color,
    selected_cell_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            row_fg: tailwind::SLATE.c200,
            selected_row_style_fg: color.c400,
            selected_column_style_fg: color.c400,
            selected_cell_style_fg: color.c600,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
        }
    }
}

/// The main application which holds the state and logic of the application.
pub struct App {
    state: TableState,
    items: Vec<Vec<String>>,
    colors: TableColors,
    placement: Vec<usize>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            state: TableState::default().with_selected(0),
            items: vec![
                vec!["1".into(), "2".into(), "3".into()],
                vec!["4".into(), "5".into(), "6".into()],
                vec!["7".into(), "8".into(), "9".into()],
            ],
            colors: TableColors::new(&PALETTES[0]),
            placement: vec![1, 1],
        }
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.placement[0] = i;
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.placement[0] = i;
    }

    pub fn next_column(&mut self) {
        if self.placement[1] == 2 {
            self.placement[1] = 0
        } else {
            self.placement[1] += 1
        };
    }

    pub fn previous_column(&mut self) {
        if self.placement[1] == 0 {
            self.placement[1] = 2
        } else {
            self.placement[1] -= 1
        };
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            // Handles the key events and updates the state of [`App`].
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match (key.modifiers, key.code) {
                        (_, KeyCode::Char('q') | KeyCode::Esc)
                        | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                            return Ok(());
                        }
                        (_, KeyCode::Char('s') | KeyCode::Down) => self.next_row(),
                        (_, KeyCode::Char('w') | KeyCode::Up) => self.previous_row(),
                        (_, KeyCode::Char('d') | KeyCode::Right) => self.next_column(),
                        (_, KeyCode::Char('a') | KeyCode::Left) => self.previous_column(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let min_width = 30;
        let min_height = 10;

        if area.width < min_width || area.height < min_height {
            let block = Paragraph::new("Terminal size too small.\nMinimum size is 30x10.")
                .centered()
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::Red));
            frame.render_widget(
                block,
                center(area, Constraint::Percentage(100), Constraint::Length(2)),
            );
        } else {
            let (title_area, layout) = calculate_layout(area);

            // handle the cell placements
            self.state.select(Some(self.placement[0]));
            self.state.select_column(Some(self.placement[1]));

            // render ui elements
            self.render_title(frame, title_area);
            self.render_table(frame, layout);
        }
    }

    fn render_title(&mut self, frame: &mut Frame, area: Rect) {
        let title = Paragraph::new("You VS Bot").centered();

        frame.render_widget(title, area);
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);
        let selected_col_style = Style::default().fg(self.colors.selected_column_style_fg);
        let selected_cell_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_cell_style_fg);

        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => self.colors.normal_row_color,
                _ => self.colors.alt_row_color,
            };
            data.into_iter()
                .map(|content| Cell::from(Text::from(format!("\n{content}")).centered()))
                .collect::<Row>()
                .style(Style::new().fg(self.colors.row_fg).bg(color))
                .height(3)
        });

        let t = Table::new(
            rows,
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ],
        )
        .row_highlight_style(selected_row_style)
        .column_highlight_style(selected_col_style)
        .cell_highlight_style(selected_cell_style)
        // .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]))
        .bg(self.colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(t, area, &mut self.state);
    }
}

fn calculate_layout(area: Rect) -> (Rect, Rect) {
    let main_layout = Layout::vertical([Constraint::Max(1), Constraint::Max(9)]).flex(Flex::Center);
    let [title_area, main_area] = main_layout.areas(area);
    (
        center(title_area, Constraint::Length(30), Constraint::Length(1)),
        center(main_area, Constraint::Length(30), Constraint::Length(9)),
    )
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
