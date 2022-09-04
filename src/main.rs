use finance_tool::{
    app::FinanceClient,
    CURRENT_CONTENT, SEARCH_STRING,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
    Terminal,
};

/// Select Market
///
enum Market {}

/// todo! Make into real error
enum ClientError {
    IncorrectInput,
}

// const COMPANY_STR: &str = include_str!("../company_symbols.json");

// todo! Divide into four
// 1) init
// 2) read
// 3) update
// 4) draw

// #[derive(Debug)]
// struct CompanyInfo {
//     name: String,
//     symbol: String
// }

enum Window {
    ApiChoice,
    Search,
    Results,
}

// Ideal layout looks more like this:

// ┌ Company symbol  Company info  Stock symbol ─────────  │    Search for:─
// │                                                                          │
// └─News ──────────Company news  Get market───────────────│
// ┌                                    ─│                  │
// │                                                                          │
// └──────────────────────────────────────────────────────────────────────────┘

// ┌Results───────────────────────────────────────────────────────────────────┐
// │                                                                          │
// │                                                                          │
// │                                                                          │
// │                                                                          │
// │                                                                          │
// │                                                                          │
// │                                                                          │
// │                                                                          │
// │                                                                          │
// │                                                                          │
// └──────────────────────────────────────────────────────────────────────────┘

fn make_table(all_choices: Vec<Span>) -> Table<'static> {
    let all_rows = all_choices
        .chunks(3)
        .map(|choices| {
            let mut row_title_vec = vec![];
            let mut choice_iter = choices.iter();
            for title in choice_iter.by_ref() {
                row_title_vec.push(&title.content)
            }
            row_title_vec
        });

    let into_rows = all_rows
        .map(|not_yet_rows| {
            let vec_of_strings = not_yet_rows
                .into_iter()
                .map(|single_item| single_item.to_string())
                .collect::<Vec<String>>();
            Row::new(vec_of_strings)
        })
        .collect::<Vec<Row>>();

    Table::new(into_rows)
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

fn main() -> Result<(), anyhow::Error> {
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut client = FinanceClient::default();

    let stock_symbols = client.stock_symbols()?;
    client.companies = stock_symbols
        .into_iter()
        .map(|info| (info.description, info.display_symbol))
        .collect::<Vec<(String, String)>>();

    // Input
    // State change / enum, char+, char-
    // Draw

    loop {
        // Handles key events and decides what to do
        client.handle_event();
        terminal.clear().unwrap();
        terminal
            .draw(|f| {
                // First 2 big blocks
                let big_blocks = Layout::default()
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
                    .split(big_blocks[0]);

                // Api choices: top left block
                let all_choices = client.all_choices();
                let choices_block = make_table(all_choices);

                // Search window: top right block
                let search_area = Paragraph::new(SEARCH_STRING.inner().clone())
                    .block(Block::default().title("Search for:").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });

                let paragraph2 = Paragraph::new(CURRENT_CONTENT.inner().clone())
                    .block(Block::default().title("Results").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });

                f.render_widget(choices_block, api_and_search_box[0]);
                f.render_widget(search_area, api_and_search_box[1]);
                f.render_widget(paragraph2, big_blocks[1]);
            })
            .unwrap();
    }
}

// tui layout might look something like this

// FINANCE TOOL
// COMPANY DATA || Market cap || This week's news
// STOCK DATA   || One stock data || Weekly data
// SEARCH COMPANY
// Company profile
