use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use reqwest::blocking::Client;

pub const API_KEY: &str = include_str!("..\\key.txt");

struct FinanceClient {
    url: String,
    client: Client,
}

// todo: fill out struct and add serde stuff
struct CompanyInfo {
    country: String,
    currency: String,
    // marketCapitalization
    market_capitalization: String
}

impl FinanceClient {
    // todo! remove unwraps
    fn get_profile_by_isin(&self, isin: &str) {
        let text = self
            .client
            .get(format!("{}/stock/profile2?isin={isin}", self.url))
            .header("X-Finnhub-Token", API_KEY)
            .send()
            .unwrap()
            .text()
            .unwrap();
        println!("Text: {text}");
    }
}

fn main() -> crossterm::Result<()> {
    let client = FinanceClient {
        url: "https://finnhub.io/api/v1/".to_string(),
        client: Client::default(),
    };

    client.get_profile_by_isin("CA87807B1076");

    loop {
        match read()? {
            Event::Key(key_event) => {
                let KeyEvent { code, modifiers } = key_event;
                match (code, modifiers) {
                    (KeyCode::Char(c), _) if c == ' ' => println!("Pressed the spacebar"),
                    (KeyCode::Char(c), modifier)
                        if c == 's' && modifier == KeyModifiers::CONTROL =>
                    {
                        println!("Somebody is trying to save a file")
                    }
                    (KeyCode::Char(c), _) => println!("Got a char: {c}"),
                    (KeyCode::Up, _) => println!("Pressed up"),
                    (KeyCode::Down, _) => println!("Pressed down"),
                    (KeyCode::Left, _) => println!("Pressed left"),
                    (KeyCode::Right, _) => println!("Pressed right"),
                    (_, _) => {}
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(num1, num2) => {
                println!("Window has been resized to {num1}, {num2}");
            }
        }
    }
}

// tui layout might look something like this

// FINANCE TOOL
// COMPANY DATA || Market cap || This week's news
// STOCK DATA   || One stock data || Weekly data
