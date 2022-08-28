use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use finance_tool::{app::{ApiChoice, FinanceClient}};
use reqwest::blocking::Client;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
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

fn main() -> Result<(), anyhow::Error> {

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut client = FinanceClient {
        url: "https://finnhub.io/api/v1/".to_string(),
        client: Client::default(),
        search_string: String::new(),
        current_content: String::new(),
        choice: ApiChoice::CompanyProfile,
        current_market: "US".to_string(),
        companies: Vec::new()
    };

    let stock_symbols = client.stock_symbols()?;
    client.companies = stock_symbols.into_iter().map(|info| {
    (info.description,
            info.display_symbol)
    }).collect::<Vec<(String, String)>>();

    // Input
    // State change / enum, char+, char-
    // Draw

    loop {
        match read().unwrap() {
            Event::Key(key_event) => {
                let KeyEvent {
                    code, modifiers, ..
                } = key_event;
                // Typing event
                match (code, modifiers) {
                    (KeyCode::Char(c), modifier)
                        if c == 'q' && modifier == KeyModifiers::CONTROL =>
                    {
                        break;
                    }
                    (KeyCode::Char(c), _) => {
                        client.search_string.push(c);
                    }
                    (KeyCode::Esc, _) => {
                        client.search_string.clear();
                    }
                    (KeyCode::Backspace, _) => {
                        client.search_string.pop();
                    }
                    (KeyCode::Enter, _) => {
                        match client.choice {
                            ApiChoice::CompanyProfile => {
                                client.current_content = match client.company_profile() {
                                    Ok(search_result) => search_result.to_string(),
                                    Err(e) => e.to_string(),
                                }        
                            }
                            ApiChoice::GetMarket => {
                                client.current_content = client.choose_market();
                            }
                            _ => {}
                        }
                        
                    }
                    (KeyCode::Tab, _) => {
                        client.switch();
                    }
                    (_, _) => {}
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(num1, num2) => {
                println!("Window has been resized to {num1}, {num2}");
            }
            Event::Paste(_s) => {}
            _ => {}
        }
        if client.choice == ApiChoice::SymbolSearch && !client.search_string.is_empty() {
            client.current_content = client.company_search(&client.search_string);
        }
        terminal.clear().unwrap();
        let current_search_string = client.search_string.clone();
        let current_content = client.current_content.clone();
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(3)
                    .constraints(
                        [
                            Constraint::Percentage(20), // Choice enum (company search, etc.)
                            Constraint::Percentage(20), // Search string
                            Constraint::Percentage(60), // Results
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let block1 = Block::default()
                    .title(client.all_choices())
                    .borders(Borders::ALL);
                f.render_widget(block1, chunks[0]);

                let block2 = Block::default().title("Search for:").borders(Borders::ALL);
                let paragraph1 = Paragraph::new(current_search_string)
                    .block(block2)
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });
                f.render_widget(paragraph1, chunks[1]);

                let block3 = Block::default().title("Results").borders(Borders::ALL);
                let paragraph2 = Paragraph::new(current_content)
                    .block(block3)
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });
                f.render_widget(paragraph2, chunks[2]);
            })
            .unwrap();
    }
    Ok(())
}

// tui layout might look something like this

// FINANCE TOOL
// COMPANY DATA || Market cap || This week's news
// STOCK DATA   || One stock data || Weekly data
// SEARCH COMPANY
// Company profile
