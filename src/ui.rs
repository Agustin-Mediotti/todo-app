use crate::app::{App, CurrentScreen};
use crate::banner::BANNER;
use model::util::is_completed;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Style};
use ratatui::symbols::{self, border};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Clear, List, Paragraph, Wrap};
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

    if let CurrentScreen::Help = app.current_screen {
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::DOUBLE)
            .style(Style::default())
            .title_bottom(" Made in 🦀 by Vitto 2025 ")
            .title_style(Style::default().fg(Color::Gray))
            .title_alignment(ratatui::layout::Alignment::Center);

        let banner_text = Text::raw(BANNER)
            .centered()
            .style(Style::default().fg(Color::Gray));

        let help_paragraph = Paragraph::new(banner_text.clone())
            .block(popup_block)
            .wrap(Wrap { trim: false })
            .centered();

        let area = center(
            frame.area(),
            Constraint::Length(banner_text.width() as u16 + 3),
            Constraint::Length(banner_text.height() as u16 + 2),
        );
        frame.render_widget(Clear, area);
        frame.render_widget(help_paragraph, area);
    }

    if let CurrentScreen::Exiting = app.current_screen {
        let popup_block = Block::default()
            .title("Y/N")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let exit_text = Text::styled(
            "Are you sure you want to exit?",
            Style::default().fg(Color::Red),
        );

        let exit_paragraph = Paragraph::new(exit_text.clone())
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = center(
            frame.area(),
            Constraint::Percentage(30),
            Constraint::Percentage(10),
        );
        frame.render_widget(Clear, area);
        frame.render_widget(exit_paragraph, area);
    }

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
            .map(|task| task.description().clone() + " " + is_completed(task.completed()).as_str())
            .collect::<Vec<String>>()
    } else {
        app.tasks
            .iter()
            .filter(|x| x.completed() == false)
            .map(|task| task.description().clone() + " " + is_completed(task.completed()).as_str())
            .collect::<Vec<String>>()
    };

    let footer_text = {
        match app.current_screen {
            CurrentScreen::Main => Paragraph::new(Line::from(vec![
                " Done ".into(),
                "<Enter>".red().bold(),
                " Show/Hide ".into(),
                "<W>".red().bold(),
                " About ".into(),
                "<H>".red().bold(),
                " Quit ".into(),
                "<Q> ".red().bold(),
            ])),
            CurrentScreen::Editing => Paragraph::new(Line::from(vec![
                " Done ".into(),
                "<Enter>".red().bold(),
                " About ".into(),
                "<H>".red().bold(),
                " Quit ".into(),
                "<Q> ".red().bold(),
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
                .title(Line::from(" Tasks ".bold()).left_aligned())
                .border_set(border::ROUNDED)
                .borders(Borders::ALL),
        )
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(true);

    frame.render_stateful_widget(list, chunks[1], &mut app.state);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
