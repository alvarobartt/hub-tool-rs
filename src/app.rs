use crate::registry::DockerRegistry;
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
    DefaultTerminal,
};

pub struct App {
    containers: Containers,
    should_quit: bool,
}

struct Containers {
    items: Vec<String>,
    state: ListState,
}

impl App {
    pub async fn new(registry: DockerRegistry) -> Self {
        let items = registry.list_repositories().await.unwrap_or(vec![]);
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            containers: Containers { items, state },
            should_quit: false,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| {
                let mut state = ListState::default();
                frame.render_stateful_widget(&mut self, frame.area(), &mut state);
            })?;
            self.handle_event(&event::read()?);
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                    KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                    _ => {}
                }
            }
        }
    }

    fn select_next(&mut self) {
        let i = match self.containers.state.selected() {
            Some(i) => {
                if i >= self.containers.items.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.containers.state.select(Some(i));
    }

    fn select_previous(&mut self) {
        let i = match self.containers.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.containers.state.select(Some(i));
    }
}

impl StatefulWidget for &mut App {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);

        let items: Vec<ListItem> = self
            .containers
            .items
            .iter()
            .map(|i| ListItem::new(i.as_str()))
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Containers").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        StatefulWidget::render(list, layout[0], buf, &mut self.containers.state);

        let tooltip = Paragraph::new(Line::from(vec![
            Span::styled("j/↓", Style::default().fg(Color::Yellow)),
            Span::raw(": next, "),
            Span::styled("k/↑", Style::default().fg(Color::Yellow)),
            Span::raw(": previous, "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": quit"),
        ]))
        .block(Block::default().borders(Borders::ALL));

        Widget::render(tooltip, layout[1], buf);
    }
}
