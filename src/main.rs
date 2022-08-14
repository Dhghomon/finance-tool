use std::collections::BTreeMap;

use anyhow::{Context, Error};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

pub const API_KEY: &str = include_str!("..\\key.txt");

struct FinanceClient {
    url: String,
    client: Client,
    search_string: String,   // push + pop // MSFT
    current_content: String, // Results etc. of searches
    choice: ApiChoice,
}

impl FinanceClient {
    fn switch(&mut self) {
        self.choice = match self.choice {
            ApiChoice::SymbolSearch => ApiChoice::CompanyInfo,
            ApiChoice::CompanyInfo => ApiChoice::SymbolSearch,
        }
    }
    fn all_choices(&self) -> Vec<Span<'static>> {
        use ApiChoice::*; // SybolSearch
        let choices = vec![format!("{:?}", SymbolSearch), format!("{:?}", CompanyInfo)];

        choices
            .into_iter()
            .map(|choice_string| {
                let current_choice = format!("{:?}", self.choice);
                if choice_string == current_choice {
                    Span::styled(
                        format!("{choice_string} "),
                        Style::default().fg(Color::LightYellow),
                    )
                } else {
                    Span::raw(format!("{choice_string} "))
                }
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApiChoice {
    SymbolSearch,
    CompanyInfo,
}

// Style::default()
//     .fg(Color::Black)
//     .bg(Color::Green)
//     .add_modifier(Modifier::ITALIC | Modifier::BOLD);

impl std::fmt::Display for ApiChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ApiChoice::*;
        let output = match self {
            SymbolSearch => "Company symbol",
            CompanyInfo => "Company info",
        };
        write!(f, "{}", output)
    }
}

/// Serialize = into JSON
///
/// Deserialize = into Rust type
#[derive(Debug, Serialize, Deserialize)]
struct CompanyInfo {
    country: String,
    currency: String,
    exchange: String,
    #[serde(rename = "finnhubIndustry")]
    industry: String,
    ipo: String,
    #[serde(rename = "marketCapitalization")]
    market_capitalization: f64,
    name: String,
    phone: String,
    #[serde(rename = "shareOutstanding")]
    shares_outstanding: f64,
    ticker: String,
    weburl: String,
}

impl std::fmt::Display for CompanyInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let CompanyInfo {
            country,
            industry,
            currency,
            exchange,
            ipo,
            market_capitalization,
            name,
            phone,
            shares_outstanding,
            ticker,
            weburl,
        } = self;

        let company_info = format!(
            "
Company name: {name}
Country: {country}
Currency: {currency}
Exchange: {exchange}
Industry: {industry}
Ipo: {ipo}
Market capitalization: {market_capitalization}
Ticker: {ticker}
Shares: {shares_outstanding}
Phone: {phone}
Url: {weburl}
"
        );
        write!(f, "{}", company_info)
    }
}

/// todo! Make into real error
enum ClientError {
    IncorrectInput,
}

impl FinanceClient {
    // todo! remove unwraps
    fn get_profile_by_symbol(&self) -> Result<String, Error> {
        let response = self
            .client
            .get(format!(
                "{}/stock/profile2?symbol={}",
                self.url, self.search_string
            ))
            .header("X-Finnhub-Token", API_KEY)
            .send()
            .with_context(|| "Couldn't send via client")?;
        let text = response.text().with_context(|| "No text for some reason")?;
        let company_info: CompanyInfo = serde_json::from_str(&text).with_context(|| {
            format!(
                "Couldn't deserialize {} into CompanyInfo struct.\nText from Finnhub: '{text}'",
                self.search_string
            )
        })?;
        Ok(company_info.to_string())
    }
}

fn company_search(needle: &str, haystack: &Vec<(&str, &str)>) -> String {
    haystack.iter().filter_map(|(company_name, company_symbol)| {
        if company_name.contains(needle) {
            Some(format!("{}: {}\n", company_symbol, company_name))
        } else {
            None
        }
    }).collect::<String>()
}

const COMPANY_STR: &str = include_str!("../company_symbols.json");

fn main() -> Result<(), anyhow::Error> {
    let companies = serde_json::from_str::<BTreeMap<&str, &str>>(COMPANY_STR)
        .unwrap()
        .into_iter()
        .map(|(key, value)| (key, value))
        .collect::<Vec<(_, _)>>();

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut client = FinanceClient {
        url: "https://finnhub.io/api/v1/".to_string(),
        client: Client::default(),
        search_string: String::new(),
        current_content: String::new(),
        choice: ApiChoice::CompanyInfo,
    };

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
                        client.current_content = match client.get_profile_by_symbol() {
                            Ok(search_result) => search_result,
                            Err(e) => e.to_string(),
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
        if client.choice == ApiChoice::SymbolSearch {
            client.current_content = company_search(&client.search_string, &companies);
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
