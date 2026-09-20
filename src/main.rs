use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::palette::tailwind::{BLUE, GREEN, SLATE};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{
    Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
    StatefulWidget, Widget, Wrap,
};
use ratatui::{DefaultTerminal, symbols};

use ders::{Output, PathMatch};

const TODO_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::default().run(terminal))
}

struct App {
    should_exit: bool,
    current_tab: Tabs,
    content: OutputState,
}

#[derive(Default)]
struct OutputState {
    output: Output,
    state: ListState,
}

#[derive(Debug)]
enum Tabs {
    // Stdin,
    Stdout,
    // Stderr,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_exit: false,
            current_tab: Tabs::Stdout,
            content: OutputState::default(),
        }
    }
}

// impl FromIterator<(Status, &'static str, &'static str)> for TodoList {
//     fn from_iter<I: IntoIterator<Item = (Status, &'static str, &'static str)>>(iter: I) -> Self {
//         let items = iter
//             .into_iter()
//             .map(|(status, todo, info)| TodoItem::new(status, todo, info))
//             .collect();
//         let state = ListState::default();
//         Self { items, state }
//     }
// }

// impl TodoItem {
//     fn new(status: Status, todo: &str, info: &str) -> Self {
//         Self {
//             status,
//             todo: todo.to_string(),
//             info: info.to_string(),
//         }
//     }
// }

impl App {
    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            // KeyCode::Char('h') | KeyCode::Left => self.select_tab_next(),
            // KeyCode::Char('l') | KeyCode::Right => self.select_tab_prev(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Enter => {
                self.edit_file();
            }
            _ => {}
        }
    }

    fn edit_file(&mut self) {
        self.select_first();
    }

    // fn select_tab_next(&mut self) {
    //     self.todo_list.state.select_next();
    // }
    // fn select_tab_prev(&mut self) {
    //     self.todo_list.state.select_previous();
    // }

    fn select_next(&mut self) {
        self.content.state.select_next();
    }
    fn select_prev(&mut self) {
        self.content.state.select_previous();
    }

    const fn select_first(&mut self) {
        self.content.state.select_first();
    }

    const fn select_last(&mut self) {
        self.content.state.select_last();
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [header_area, content_area, footer_area] = area.layout(&main_layout);

        let content_layout = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]);
        let [list_area, item_area] = content_area.layout(&content_layout);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);
    }
}

/// Rendering logic for the app
impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Showing only stdout for now! ")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("This list is only showing matches for stdout!").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        let items: Vec<ListItem> = self
            .content
            .output
            .o_stdout
            .matches
            .iter()
            .enumerate()
            .map(|(idx, path_match_item)| {
                let color = alternate_colors(idx);
                ListItem::from(path_match_item).bg(color)
            })
            .collect();

        // Iterate through all elements in the `items` and stylize them.
        // let items: Vec<ListItem> = self
        //     .todo_list
        //     .items
        //     .iter()
        //     .enumerate()
        //     .map(|(i, todo_item)| {
        //         let color = alternate_colors(i);
        //         ListItem::from(todo_item).bg(color)
        //     })
        //     .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.content.state);
    }

    // fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
    //     // We get the info depending on the item's state.
    //     let info = if let Some(i) = self.todo_list.state.selected() {
    //         match self.todo_list.items[i].status {
    //             Status::Completed => format!("✓ DONE: {}", self.todo_list.items[i].info),
    //             Status::Todo => format!("☐ TODO: {}", self.todo_list.items[i].info),
    //         }
    //     } else {
    //         "Nothing selected...".to_string()
    //     };
    //
    //     // We show the list item's info under the list in this paragraph
    //     let block = Block::new()
    //         .title(Line::raw("TODO Info").centered())
    //         .borders(Borders::TOP)
    //         .border_set(symbols::border::EMPTY)
    //         .border_style(TODO_HEADER_STYLE)
    //         .bg(NORMAL_ROW_BG)
    //         .padding(Padding::horizontal(1));
    //
    //     // We can now render the item info
    //     Paragraph::new(info)
    //         .block(block)
    //         .fg(TEXT_FG_COLOR)
    //         .wrap(Wrap { trim: false })
    //         .render(area, buf);
    // }
}

const fn alternate_colors(i: usize) -> Color {
    if i.is_multiple_of(2) {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}
