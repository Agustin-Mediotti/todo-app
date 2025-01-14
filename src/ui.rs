use crate::app::{App, CurrentScreen};
use crate::banner::BANNER;
use model::util::is_completed;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Style};
use ratatui::symbols::{self, border};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, Padding, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let throbber_widget = throbber_widgets_tui::Throbber::default()
        .throbber_style(
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Red)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .throbber_set(throbber_widgets_tui::BRAILLE_SIX)
        .use_type(throbber_widgets_tui::WhichUse::Spin);

    if let true = app.loading {
        frame.render_stateful_widget(throbber_widget, chunks[0], &mut app.throbber_state);
    }

    let items: Vec<String> = if app.show_done {
        app.tasks
            .iter()
            .map(|task| {
                is_completed(task.completed()).as_str().to_owned()
                    + " "
                    + task.description().as_str()
            })
            .collect::<Vec<String>>()
    } else {
        app.tasks
            .iter()
            .filter(|x| x.completed() == false)
            .map(|task| {
                is_completed(task.completed()).as_str().to_owned()
                    + " "
                    + task.description().as_str()
            })
            .collect::<Vec<String>>()
    };

    let footer_text = {
        match app.current_screen {
            CurrentScreen::Main => Paragraph::new(Line::from(vec![
                " Select ".into(),
                "<Enter>".red().bold(),
                " Show/Hide ".into(),
                "<W>".red().bold(),
                " About ".into(),
                "<H>".red().bold(),
                " Quit ".into(),
                "<Q> ".red().bold(),
            ])),
            CurrentScreen::Editing => Paragraph::new(Line::from(vec![
                " Done Editing ".into(),
                "<Enter> ".red().bold(),
                " Mark as Done ".into(),
                "<Tab>".red().bold(),
                " Back ".into(),
                "<Esc> ".red().bold(),
            ])),
            CurrentScreen::Help => {
                Paragraph::new(Line::from(vec!["Back ".into(), "<Q> ".red().bold()]))
            }
            CurrentScreen::Exiting => Paragraph::new(Line::from(vec![
                " Back ".into(),
                "<N/ESC>".red().bold(),
                " Quit ".into(),
                "<Q/Y> ".red().bold(),
            ])),
        }
    };

    frame.render_widget(footer_text, chunks[2]);

    let list = List::new(items)
        .block(
            Block::bordered()
                .border_set(border::ROUNDED)
                .borders(Borders::ALL),
        )
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(true)
        .highlight_spacing(ratatui::widgets::HighlightSpacing::WhenSelected);

    frame.render_stateful_widget(list, chunks[1], &mut app.state);

    let nick = vec![
        Span::styled(" 🦀 by ", Style::default()),
        Span::styled("N37CR347UR3", Style::default()),
        Span::styled(" - 2025 ", Style::default()),
    ];

    if let CurrentScreen::Help = app.current_screen {
        let popup_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_set(symbols::border::PLAIN)
            .style(Style::default())
            .title_bottom(nick)
            .title_style(Style::default().fg(Color::Gray))
            .title_alignment(ratatui::layout::Alignment::Center);

        let banner_text = Text::raw(BANNER)
            .centered()
            .style(Style::default().fg(Color::Yellow));

        let help_paragraph = Paragraph::new(banner_text.clone())
            .block(popup_block)
            .wrap(Wrap { trim: false })
            .centered();

        let area = center(
            frame.area(),
            Constraint::Length(banner_text.width() as u16 + 3),
            Constraint::Length(banner_text.height() as u16 + 2),
        );
        frame.render_widget(help_paragraph, area);
    }

    if let CurrentScreen::Editing = app.current_screen {
        let popup_block = Block::default()
            .title(" ".to_owned() + &app.tasks[app.state.selected().unwrap()].description() + " ")
            .title_style(Style::default().fg(Color::LightYellow))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .style(Style::default())
            .padding(Padding::horizontal(2));

        let wrapped_lines = wrap_text(&app.body_input, ((frame.area().width * 50) / 100) as usize);
        let text_height = wrapped_lines.len() as u16 + 2;

        let wrapped_text = wrapped_lines
            .into_iter()
            .map(|line| Line::from(Span::raw(line)))
            .collect::<Vec<_>>();

        let area = center(
            frame.area(),
            Constraint::Percentage(50),
            Constraint::Length(text_height),
        );

        let editing_text = Paragraph::new(wrapped_text)
            .block(popup_block)
            .left_aligned();

        frame.render_widget(editing_text, area);
    }

    if let CurrentScreen::Exiting = app.current_screen {
        let popup_block = Block::default()
            .title_bottom(Line::from(" Y/N ").right_aligned().fg(Color::LightYellow))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .style(Style::default())
            .padding(Padding::vertical(2));

        let exit_text = Text::styled(
            "Are you sure you want to exit?",
            Style::default().bold().fg(Color::Red),
        )
        .alignment(ratatui::layout::Alignment::Center);

        let exit_paragraph = Paragraph::new(exit_text.clone())
            .block(popup_block)
            .centered()
            .wrap(Wrap { trim: false });

        let area = center(
            frame.area(),
            Constraint::Length(exit_text.width() as u16 + 4),
            Constraint::Length(exit_text.height() as u16 + 6),
        );
        frame.render_widget(Clear, frame.area());
        frame.render_widget(exit_paragraph, area);
    }
}
fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

fn wrap_text(input: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for ch in input.chars() {
        if current_line.len() + 1 > max_width {
            lines.push(current_line);
            current_line = String::new();
        }
        current_line.push(ch);
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
