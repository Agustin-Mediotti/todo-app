use crate::app::App;
use model::util::is_completed;
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List};
use ratatui::Frame;

pub fn render(app: &mut App, frame: &mut Frame) {
    let full = throbber_widgets_tui::Throbber::default()
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
        .throbber_style(
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Red)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .throbber_set(throbber_widgets_tui::BRAILLE_SIX)
        .use_type(throbber_widgets_tui::WhichUse::Spin);
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

    let instructions = Line::from(vec![
        " Done ".into(),
        "<Enter>".red().bold(),
        " Show/Hide ".into(),
        "<Ctrl + H>".red().bold(),
        " Help ".into(),
        "<Ctrl + ?>".red().bold(), // TODO: make a prompt displaying info
        " Quit ".into(),
        "<Q> ".red().bold(),
    ]);

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(Line::from(" Tasks ".bold()).centered())
                .border_set(border::ROUNDED)
                .title_bottom(instructions.centered())
                .borders(Borders::ALL),
        )
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(true);

    frame.render_stateful_widget(list, frame.area(), &mut app.state);
    frame.render_stateful_widget(full, frame.area(), &mut app.throbber_state);
}
