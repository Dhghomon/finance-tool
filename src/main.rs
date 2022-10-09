use std::io::Stdout;

use finance_tool::{app::State, Window, CURRENT_CONTENT, SEARCH_STRING};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
    Terminal,
};

// Company news
// Small window for error / debug messages

/// Select Market
///
enum Market {}

/// todo! Make into real error
enum ClientError {
    IncorrectInput,
}

// const COMPANY_STR: &str = include_str!("../company_symbols.json");

fn make_table(all_choices: Vec<Span>) -> Table {
    let all_rows = all_choices
        .chunks(3)
        .map(|not_yet_row| {
            let as_vec = not_yet_row.to_vec();
            Row::new(as_vec)
        })
        .collect::<Vec<_>>();

    Table::new(all_rows)
        // You can set the style of the entire Table.
        .style(Style::default().fg(Color::White))
        // As any other widget, a Table can be wrapped in a Block.
        .block(Block::default().title("Api choices").borders(Borders::ALL))
        // Columns widths are constrained in the same way as Layout...
        .widths(&[
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        // ...and they can be separated by a fixed spacing.
        .column_spacing(1)
        // If you wish to highlight a row in any specific way when it is selected...
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        // ...and potentially show a symbol in front of the selection.
        .highlight_symbol(">>")
}

pub fn draw_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>, state: &State) {

    terminal
        .draw(|f| {
            // First 2 big blocks
            let top_and_bottom = Layout::default()
                .direction(Direction::Vertical)
                .margin(3)
                .constraints(
                    [
                        Constraint::Percentage(40), // api and search box
                        Constraint::Percentage(60), // Results
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // 2 Rects
            let api_and_search_box = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(top_and_bottom[0]);

            let highlighted = Style::default().fg(Color::LightYellow);
            let unhighlighted = Style::default();
            let api_choice_border_style = match state.current_window {
                Window::ApiChoice => highlighted,
                Window::Results => unhighlighted,
            };

            let results_border_style = match state.current_window {
                Window::Results => highlighted,
                Window::ApiChoice => unhighlighted,
            };

            // Api choices: top left block
            let api_choices = make_table(state.all_choices()).block(
                Block::default()
                    .title("Api search")
                    .borders(Borders::ALL)
                    .border_style(api_choice_border_style),
            );

            // Search window: top right block
            let search_area = Paragraph::new(SEARCH_STRING.inner().clone())
                .block(Block::default().title("Search for:").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            let results = Paragraph::new(CURRENT_CONTENT.inner().clone())
                .block(
                    Block::default()
                        .title("Results")
                        .borders(Borders::ALL)
                        .border_style(results_border_style),
                )
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            f.render_widget(api_choices, api_and_search_box[0]);
            f.render_widget(search_area, api_and_search_box[1]);
            f.render_widget(results, top_and_bottom[1]);
        })
        .unwrap();
}

fn main() -> Result<(), anyhow::Error> {
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = State::default();

    state.stock_symbols_init()?;
    terminal.clear()?;
    draw_terminal(&mut terminal, &state);

    loop {
        // Handles key events and decides what to do
        state.handle_event();
        terminal.clear()?;
        draw_terminal(&mut terminal, &state);
    }
}
