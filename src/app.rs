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
use serde_json::json;

pub struct App {
    containers: Containers,
    registry: DockerRegistry,
    should_quit: bool,
}

struct Containers {
    items: Vec<Container>,
    item_enter: bool,
    list_state: ListState,
}

struct Container {
    url: String,
    info: Option<serde_json::Value>,
}

impl App {
    pub async fn new(registry: DockerRegistry) -> Self {
        let items = registry
            .list_repositories()
            .await
            .unwrap_or(vec![])
            .iter()
            .map(|url| Container {
                url: url.to_string(),
                info: None,
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            containers: Containers {
                items,
                item_enter: false,
                list_state,
            },
            registry,
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
                    KeyCode::Enter => self.display_info(),
                    _ => {}
                }
            }
        }
    }

    fn select_next(&mut self) {
        let i = match self.containers.list_state.selected() {
            Some(i) => {
                if i >= self.containers.items.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.containers.list_state.select(Some(i));
    }

    fn select_previous(&mut self) {
        let i = match self.containers.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.containers.list_state.select(Some(i));
    }

    fn display_info(&mut self) {
        match self.containers.list_state.selected() {
            Some(i) => match self.containers.items.get_mut(i) {
                Some(value) => {
                    self.containers.item_enter = true;
                    value.info = Some(json!({"os/arch": "linux/amd64"}));
                    // TODO: use the API call instead
                    // let manifest = self
                    //     .registry
                    //     .get_manifest(value.url.as_str(), "latest")
                    //     .await?;
                }
                None => self.containers.item_enter = false,
            },
            None => self.containers.item_enter = false,
        }
    }
}

impl StatefulWidget for &mut App {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let layout = if self.containers.item_enter {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area)
        } else {
            Layout::default()
                // .direction(Direction::Vertical)
                .constraints([Constraint::Min(3)])
                .split(area)
        };

        let items: Vec<ListItem> = self
            .containers
            .items
            .iter()
            .map(|i| ListItem::new(i.url.as_str()))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(
                        "Available repositories in {}",
                        self.registry.url.as_str()
                    ))
                    .borders(Borders::ALL)
                    .title_bottom("j/k or ↓/↑ to scroll, q or esc to quit"),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        StatefulWidget::render(list, layout[0], buf, &mut self.containers.list_state);

        if self.containers.item_enter {
            match self.containers.list_state.selected() {
                Some(i) => match self.containers.items.get(i) {
                    Some(v) => {
                        let info = Paragraph::new(Line::from(v.info.clone().unwrap().to_string()))
                            .block(Block::default().borders(Borders::ALL));
                        Widget::render(info, layout[1], buf);
                    }
                    None => (),
                },
                None => (),
            }
        }
    }
}
