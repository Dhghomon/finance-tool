use std::fmt::Debug;

pub const API_KEY: &str = include_str!("..\\key.txt");
pub const FINNHUB_URL: &str = "https://finnhub.io/api/v1";
pub const EXCHANGE_CODES: [&str; 72] = [
    "AS", "AT", "AX", "BA", "BC", "BD", "BE", "BK", "BO", "BR", "CA", "CN", "CO", "CR", "DB", "DE",
    "DU", "F", "HE", "HK", "HM", "IC", "IR", "IS", "JK", "JO", "KL", "KQ", "KS", "L", "LN", "LS",
    "MC", "ME", "MI", "MU", "MX", "NE", "NL", "NS", "NZ", "OL", "PA", "PM", "PR", "QA", "RG", "SA",
    "SG", "SI", "SN", "SR", "SS", "ST", "SW", "SZ", "T", "TA", "TL", "TO", "TW", "TWO", "US", "V",
    "VI", "VN", "VS", "WA", "HA", "SX", "TG", "SC",
];

#[derive(PartialEq, Eq, Debug)]
pub enum Window {
    ApiChoice,
    Results,
}

pub mod app {
    use std::{
        fmt::Debug,
        fs::File,
        io::{Stdout, Write},
        sync::mpsc::{Receiver, SyncSender},
    };

    use anyhow::{Context, Error, bail};
    use chrono::{Months, Utc, TimeZone};
    use crossterm::event::{read, Event, KeyCode, KeyEvent};
    use reqwest::{
        blocking::Client,
        header::{HeaderMap, HeaderValue},
    };
    use serde::de::DeserializeOwned;
    use tui::{
        backend::CrosstermBackend,
        layout::{Alignment, Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::Span,
        widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
        Terminal,
    };

    use crate::{
        api::{CompanyProfile, MarketNews, StockSymbol, CompanyNews},
        Window, API_KEY, EXCHANGE_CODES, FINNHUB_URL,
    };

    pub fn handle_event(sender: &SyncSender<Command>) {
        match read().unwrap() {
            Event::Key(key_event) => {
                let KeyEvent {
                    code, modifiers, ..
                } = key_event;
                // Typing event
                match (code, modifiers) {
                    (KeyCode::Char(c), _) => {
                        sender.send(Command::Char(c)).unwrap();
                    }
                    (KeyCode::Esc, _) => {
                        sender.send(Command::Esc).unwrap();
                    }
                    (KeyCode::Backspace, _) => {
                        sender.send(Command::Backspace).unwrap();
                    }
                    (KeyCode::Enter, _) => {
                        sender.send(Command::Enter).unwrap();
                    }

                    //     ApiChoice::GetMarket => {
                    //         *CURRENT_CONTENT.inner() = self.choose_market();
                    //     }

                    (KeyCode::Left, _) => {
                        sender.send(Command::Left).unwrap();
                    }
                    (KeyCode::Right, _) => {
                        sender.send(Command::Right).unwrap();
                    }
                    (KeyCode::Tab, _) => {
                        sender.send(Command::Tab).unwrap();
                    }
                    _ => {}
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(num1, num2) => {
                //println!("Window has been resized to {num1}, {num2}");
            }
            Event::Paste(_s) => {}
            _ => {}
        }
    }

    #[derive(Debug)]
    pub struct TotalApiChoices {
        pub all_apis: Vec<ApiChoice>,
        pub current_index: usize,
    }

    impl Default for TotalApiChoices {
        fn default() -> Self {
            Self {
                all_apis: vec![
                    ApiChoice::SymbolSearch,
                    ApiChoice::CompanyProfile,
                    ApiChoice::StockSymbol,
                    ApiChoice::MarketNews,
                    ApiChoice::CompanyNews,
                    ApiChoice::GetMarket,
                ],
                current_index: 0,
            }
        }
    }

    impl TotalApiChoices {
        pub fn left(&mut self) {
            self.current_index = match self.current_index.checked_sub(1) {
                Some(okay_number) => okay_number,
                None => self.all_apis.len() - 1,
            };
        }
        pub fn right(&mut self) {
            let next_number = self.current_index + 1;
            self.current_index = if next_number > (self.all_apis.len() - 1) {
                0
            } else {
                next_number
            };
        }
        pub fn current_api(&self) -> ApiChoice {
            self.all_apis[self.current_index]
        }
    }

    #[derive(Debug)]
    pub struct FinanceClient {
        pub client: Client,
        pub sender: SyncSender<Command>,
        pub receiver: Receiver<ApiCommand>,
    }

    #[derive(Debug)]
    pub struct State {
        pub current_window: Window,
        pub api_choices: TotalApiChoices,
        pub current_market: String,
        pub companies: Vec<String>,
        pub current_content: String,
        pub search_string: String,
        pub api_sender: SyncSender<ApiCommand>,
        pub receiver: Receiver<Command>,
    }

    pub enum Command {
        Backspace,
        Char(char),
        CompanyInfo(Vec<StockSymbol>),
        //CompanyNews,
        Enter,
        Esc,
        Left,
        // Gets something that needs to go in the result window
        ResultWindow(String),
        StockSymbols(Result<Vec<StockSymbol>, Error>),
        Right,
        Tab,
    }

    pub enum ApiCommand {
        SingleRequest,
        MultiRequest,
        // name of company to get profile
        CompanyNews(String),
        CompanyProfile(String),
        GetText,
        // current_market as a String
        StockSymbols(String),
        MarketNews,
    }

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

    impl State {
        pub fn receive_command(&mut self) {
            let command = self.receiver.recv().unwrap();
            match command {
                Command::Backspace => {
                    self.search_string.pop();
                }
                Command::Char(c) => {
                    self.search_string.push(c);
                }
                Command::Enter => match self.api_choice() {
                    ApiChoice::CompanyProfile => {
                        self.api_sender
                            .send(ApiCommand::CompanyProfile(self.search_string.clone()))
                            .unwrap();
                    }
                    ApiChoice::GetMarket => {
                        todo!();
                    }
                    ApiChoice::MarketNews => {
                        self.send_command(ApiCommand::MarketNews);
                    }
                    ApiChoice::CompanyNews => {
                        self.send_command(ApiCommand::CompanyNews(self.search_string.clone()));
                    }
                    _ => {}
                },
                Command::Esc => {
                    self.search_string.clear();
                }
                Command::Left => {
                    if self.current_window == Window::ApiChoice {
                        self.api_choices.left();
                    }
                }
                Command::ResultWindow(s) => {
                    self.current_content = s;
                }
                Command::Right => {
                    if self.current_window == Window::ApiChoice {
                        self.api_choices.right();
                    }
                }
                Command::StockSymbols(stock_symbols_res) => {
                    match stock_symbols_res {
                        Ok(stock_symbols) => {
                            let as_companies = stock_symbols.into_iter().map(|stock_symbols| {
                                format!("{} : {}", stock_symbols.description, stock_symbols.symbol)
                            }).collect::<Vec<String>>();
                            self.companies = as_companies;
                        },
                        Err(e) => {
                            self.current_content = e.to_string();
                        }
                    }
                }
                Command::Tab => {
                    self.switch_window();
                }
                Command::CompanyInfo(company_info) => {
                    //self.companies = company_info.clone();

                    let mut file = File::create("company_symbols.txt").unwrap();
                    let num = self
                        .companies
                        .iter()
                        .fold(0, |first, second| second.len() + first);
                    let mut output_string = String::with_capacity(num);
                    company_info.iter().for_each(|s| {
                        let s = format!("{} {}", s.display_symbol, s.description);
                        output_string.push_str(&s);
                        output_string.push('\n');
                    });
                    file.write_all(output_string.as_bytes()).unwrap();
                }
            }
        }

        pub fn check_self(&mut self) {
            // Symbol Search should happen every time the user has selected Symbol Search
            // and search_string is at least one character long
            if self.api_choice() == ApiChoice::SymbolSearch && !self.search_string.is_empty() {
                self.current_content = self.company_search(&self.search_string);
                if self.current_content.is_empty() {
                    self.current_content = "Still waiting for market info".into();
                }
            }
        }

        pub fn send_command(&mut self, command: ApiCommand) {
            self.api_sender.send(command).unwrap();
        }

        pub fn draw_terminal(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
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
                        .constraints(
                            [Constraint::Percentage(70), Constraint::Percentage(30)].as_ref(),
                        )
                        .split(top_and_bottom[0]);

                    let highlighted = Style::default().fg(Color::LightYellow);
                    let unhighlighted = Style::default();
                    let api_choice_border_style = match self.current_window {
                        Window::ApiChoice => highlighted,
                        Window::Results => unhighlighted,
                    };

                    let results_border_style = match self.current_window {
                        Window::Results => highlighted,
                        Window::ApiChoice => unhighlighted,
                    };

                    // Api choices: top left block
                    let api_choices = make_table(self.all_choices()).block(
                        Block::default()
                            .title("Api search")
                            .borders(Borders::ALL)
                            .border_style(api_choice_border_style),
                    );

                    // Search window: top right block
                    let search_area = Paragraph::new(self.search_string.clone())
                        .block(Block::default().title("Search for:").borders(Borders::ALL))
                        .style(Style::default().fg(Color::White).bg(Color::Black))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: true });

                    let results = Paragraph::new(self.current_content.clone())
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

        pub fn new(api_sender: SyncSender<ApiCommand>, receiver: Receiver<Command>) -> Self {
            Self {
                current_window: Window::ApiChoice,
                api_choices: TotalApiChoices::default(),
                current_market: "US".to_string(),
                companies: Vec::new(),
                current_content: String::new(),
                search_string: String::new(),
                api_sender,
                receiver,
            }
        }

        // todo!() turn this into Tables: 3*3 and then later 4*4
        pub fn all_choices(&self) -> Vec<Span> {
            let choices = &self.api_choices.all_apis;

            choices
                .iter()
                .enumerate()
                .map(|(index, api_name)| {
                    if self.api_choices.current_index == index {
                        Span::styled(format!("{api_name}"), Style::default().bg(Color::Gray))
                    } else {
                        Span::styled(format!("{api_name}"), Style::default().bg(Color::Black))
                    }
                })
                .collect::<Vec<_>>()
        }

        pub fn stock_symbols_init(&mut self) -> Result<(), Error> {
                    self.api_sender
                        .send(ApiCommand::StockSymbols(self.current_market.clone()))
                        .unwrap();
                    Ok(())
        }

        /// User hits enter, checks to see if market exists, if not, stay with original one
        pub fn choose_market(&mut self) {
            if self.current_market == self.search_string {
                self.current_content = format!("Already using market {}", self.search_string);
            }
            match EXCHANGE_CODES
                .iter()
                .find(|code| **code == self.search_string)
            {
                // e.g. user types "T", which is valid
                Some(good_market_code) => {
                    // todo! take this unwrap back out
                    // Add debugging window or something
                    self.api_sender
                        .send(ApiCommand::StockSymbols(good_market_code.to_string()))
                        .unwrap();
                    // let url = format!(
                    //     "{FINNHUB_URL}/stock/symbol?exchange={}",
                    //     self.current_market
                    // );
                    // let stock_symbols = self.stock_symbols(url).unwrap();
                    // // Now self.current_market is "T"
                    // self.current_market = good_market_code.to_string();
                    // self.search_string.clear();
                    // self.companies = stock_symbols
                    //     .into_iter()
                    //     .map(|info| format!("{}: {}", info.description, info.display_symbol))
                    //     .collect::<Vec<_>>();
                    // format!(
                    //     "Successfully got company info from market {}",
                    //     self.current_market
                    // )
                }
                // user types something that isn't a market
                None => {
                    self.current_content = format!("No market called {} exists", self.search_string)
                }
            }
        }

        pub fn switch_window(&mut self) {
            self.current_window = match self.current_window {
                Window::ApiChoice => Window::Results,
                Window::Results => Window::ApiChoice,
            }
        }

        pub fn api_choice(&self) -> ApiChoice {
            self.api_choices.current_api()
        }

        pub fn company_search(&self, needle: &str) -> String {
            self.companies
                .iter()
                .filter_map(|info| {
                    // company name, company symbol
                    let needle = needle.to_lowercase();
                    if info.to_lowercase().contains(&needle) {
                        Some(format!("{info}\n"))
                    } else {
                        None
                    }
                })
                .collect::<String>()
        }
    }

    /// Vec<StockSymbol>
    impl FinanceClient {
        pub fn receive_command(&self) {
            let api_command = self.receiver.recv().unwrap();
            match api_command {
                ApiCommand::StockSymbols(url) => {
                    self.stock_symbols(url).unwrap();
                }
                ApiCommand::SingleRequest => todo!(),
                ApiCommand::MultiRequest => todo!(),
                ApiCommand::CompanyNews(company_symbol) => {
                    let company_news_res = self.company_news(&company_symbol);
                    let result_window_content = match company_news_res {
                        Ok(company_stuff) => company_stuff,
                        Err(e) => e.to_string()
                    };
                    self.sender.send(Command::ResultWindow(result_window_content)).unwrap();
                }
                ApiCommand::CompanyProfile(company_name) => {
                    let company_info = self.company_profile(company_name);
                    self.sender
                        .send(Command::ResultWindow(company_info))
                        .unwrap();
                }
                ApiCommand::GetText => todo!(),
                ApiCommand::MarketNews => {

                    let market_res = match self.market_news() {
                        Ok(market_news) => market_news,
                        Err(e) => e.to_string()
                    };
                    self.sender.send(Command::ResultWindow(market_res)).unwrap();
                },
            }
        }

        pub fn new(sender: SyncSender<Command>, receiver: Receiver<ApiCommand>) -> Self {
            let mut headers = HeaderMap::new();
            headers.insert("X-Finnhub-Token", HeaderValue::from_static(API_KEY));
            let client = Client::builder().default_headers(headers).build().unwrap();

            Self {
                client,
                receiver,
                sender,
            }
        }

        pub fn single_request<T: DeserializeOwned + Debug>(
            &self,
            url: String,
            company_name: &str,
        ) -> Result<T, Error> {
            let text = self.get_text(url)?;
            let finnhub_reply: T = serde_json::from_str(&text).with_context(|| {
                format!(
                    "Couldn't deserialize {company_name} into CompanyProfile struct.\nText from Finnhub: '{text}'"
                )
            })?;
            Ok(finnhub_reply)
        }

        pub fn multi_request<T: DeserializeOwned + Debug>(
            &self,
            url: String,
            company_name: &str,
        ) -> Result<Vec<T>, Error> {
            let text = self.get_text(url)?;

            let finnhub_reply: Vec<T> = serde_json::from_str(&text).with_context(|| {
                format!(
                    "Couldn't deserialize {company_name} into CompanyProfile struct.\nText from Finnhub: '{text}'",
                )
            })?;
            Ok(finnhub_reply)
        }

        pub fn company_profile(&self, company_name: String) -> String {
            // /stock/profile?symbol=AAPL
            let url = format!("{FINNHUB_URL}/stock/profile2?symbol={company_name}");
            match self.single_request::<CompanyProfile>(url, &company_name) {
                Ok(company_profile) => company_profile.to_string(),
                Err(e) => e.to_string(),
            }
        }

        pub fn get_text(&self, url: String) -> Result<String, Error> {
            let url = format!("{url}&token={API_KEY}");
            self.client
                .get(url)
                .header("X-Finnhub-Token", API_KEY)
                .send()
                .with_context(|| "Couldn't send via client")?
                .text()
                .with_context(|| "No text for some reason")
        }

        pub fn stock_symbols(&self, current_market: String) -> Result<(), Error> {
            let url = format!("{FINNHUB_URL}/stock/symbol?exchange={current_market}");
            let text = match self.get_text(url) {
                Ok(text) => text,
                Err(e) => bail!(e.to_string())
            };
            //let mut new_file = File::create("stock_symbols.json")?;
            //write!(&mut new_file, "{}", text)?;
            let stock_symbols: Result<Vec<StockSymbol>, Error> = serde_json::from_str(&text)
                .map_err(|e| anyhow::anyhow!(format!("Couldn't make any stock symbols: {e}")));
            self.sender.send(Command::StockSymbols(stock_symbols)).unwrap();
            Ok(())
            //Ok(())
            //self.stock_symbols(current_market)
            //Ok(stock_symbols)
        }

        /// /stock/symbol?exchange=US
        // pub fn stock_symbols_init(&self, current_market: String) -> Result<Vec<StockSymbol>, Error> {
        //     let url = format!(
        //         "{FINNHUB_URL}/stock/symbol?exchange={current_market}",
        //     );
        //     let company_info = self
        //         .multi_request(url)
        //         .unwrap()
        //         .into_iter()
        //         .map(|s| format!("{}: {}\n", s.description, s.display_symbol))
        //         .collect::<Vec<String>>();

        //     let text = self.get_text(url)?;
        //     let mut new_file = File::create("stock_symbols.json")?;
        //     write!(&mut new_file, "{}", text)?;
        //     let stock_symbols: Vec<StockSymbol> = serde_json::from_str(&text).unwrap();
        //     Ok(stock_symbols)
        // }

        /// company-news?symbol=AAPL&from=2021-09-01&to=2021-09-09
        /// Required: date + symbol
        pub fn company_news(&self, company_symbol: &str) -> Result<String, Error> {
            let now = chrono::Utc::today().naive_utc();
            let six_months = Months::new(6);
            let six_months_ago = now - six_months;
            let url = format!("{FINNHUB_URL}/company-news/?symbol={company_symbol}&from={six_months_ago}&to={now}");
            let news_items: Result<Vec<CompanyNews>, Error> = self.multi_request(url, company_symbol);
            match news_items {
                Err(e) => {
                    Err(anyhow::anyhow!(format!("Couldn't get news for company {company_symbol}: {e}")))
                },
                Ok(items) if items.is_empty() => {
                    Err(anyhow::anyhow!(format!("Couldn't get news for company {company_symbol}")))
                }
                Ok(items) => {
                    Ok(items.into_iter().map(|blurb| {
                        let datetime = Utc.timestamp(blurb.datetime, 0).date_naive();
                        format!("{} {} || {}\n\n", datetime, blurb.headline, blurb.source)
                    })
                    .take(5)
                    .collect::<String>())
                }
            }
        }

        // todo! Let user decide on a topic - going with general for now
        /// news?category=general
        /// This parameter can be 1 of the following values general, forex, crypto, merger
        pub fn market_news(&self) -> Result<String, Error> {
            //news?category=general
            let url = format!("{FINNHUB_URL}/news/?category=general&minId=7178340");
            let text = self.get_text(url)?;
            let market_news: Vec<MarketNews> = serde_json::from_str(&text)?;
            let mut output_string = String::new();
            market_news
                .into_iter()
                .take(5)
                .for_each(|bit_of_news| output_string.push_str(&format!("{bit_of_news}\n")));
            Ok(output_string)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ApiChoice {
        SymbolSearch,
        CompanyProfile,
        StockSymbol,
        MarketNews,
        CompanyNews,
        GetMarket,
    }

    impl std::fmt::Display for ApiChoice {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use ApiChoice::*;
            let output = match self {
                SymbolSearch => "Symbol Search",
                CompanyProfile => "Company Profile",
                StockSymbol => "Stock Symbol",
                MarketNews => "Market News",
                CompanyNews => "Company News",
                GetMarket => "Get Market",
            };
            write!(f, "{}", output)
        }
    }
}

/// Structs and enums for the Finnhub API.
pub mod api {

    // /// Company name, company symbol
    // #[derive(Serialize, Deserialize, Debug)]
    // pub struct CompanySymbol(pub String, pub String);

    // impl Display for CompanySymbol {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         write!(f, "{}: {}", self.0, self.1)
    //     }
    // }

    // impl From<StockSymbol> for CompanySymbol {
    //     fn from(stock_symbol: StockSymbol) -> Self {
    //         Self(stock_symbol.description, stock_symbol.display_symbol)
    //     }
    // }

    /// Serialize = into JSON
    ///
    /// Deserialize = into Rust type
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CompanyProfile {
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

    impl std::fmt::Display for CompanyProfile {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let CompanyProfile {
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

    //Symbol lookup

    use chrono::{TimeZone, Utc};
    /// description": "APPLE INC",
    /// "displaySymbol": "AAPL",
    ///   "symbol": "AAPL",
    ///   "type": "Common Stock"
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SymbolLookup {
        pub description: String,
        #[serde(rename = "displaySymbol")]
        pub display_symbol: String,
        pub symbol: String,
        #[serde(rename = "type")]
        pub type_: String,
    }

    // Stock Symbol

    /// "currency": "USD",
    /// "description": "UAN POWER CORP",
    /// "displaySymbol": "UPOW",
    /// "figi": "BBG000BGHYF2",
    /// "mic": "OTCM",
    /// "symbol": "UPOW",
    /// "type": "Common Stock"
    ///
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct StockSymbol {
        pub currency: String,
        pub description: String,
        #[serde(rename = "displaySymbol")]
        pub display_symbol: String,
        pub figi: String,
        pub mic: String,
        pub symbol: String,
        #[serde(rename = "type")]
        pub type_: String,
    }

    // Market news

    /// "category": "technology",
    /// "datetime": 1596589501,
    /// "headline": "Square surges after reporting 64% jump in revenue, more customers using Cash App",
    /// "id": 5085164,
    /// "image": "https://image.cnbcfm.com/api/v1/image/105569283-1542050972462rts25mct.jpg?v=1542051069",
    /// "related": "",
    /// "source": "CNBC",
    /// "summary": "Shares of Square soared on Tuesday evening after posting better-than-expected quarterly results and strong growth in its consumer payments app.",
    /// "url": "https://www.cnbc.com/2020/08/04/square-sq-earnings-q2-2020.html"
    #[derive(Debug, Serialize, Deserialize)]
    pub struct MarketNews {
        pub category: String,
        pub datetime: i64,
        pub headline: String,
        //pub id: i64,
        //pub image: String,
        //pub related: String,
        pub source: String,
        //pub summary: String,
        //pub url: String,
    }

    impl std::fmt::Display for MarketNews {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let MarketNews {
                category,
                headline,
                source,
                datetime,
            } = self;
            // 2017-07-14
            let datetime = Utc.timestamp(*datetime, 0).date_naive();

            write!(f, "{datetime}: {category} from {source}:\n  {headline}\n")
        }
    }

    // Company News

    //   {
    //     "category": "company news",
    //     "datetime": 1569550360,
    //     "headline": "More sops needed to boost electronic manufacturing: Top govt official More sops needed to boost electronic manufacturing: Top govt official.  More sops needed to boost electronic manufacturing: Top govt official More sops needed to boost electronic manufacturing: Top govt official",
    //     "id": 25286,
    //     "image": "https://img.etimg.com/thumb/msid-71321314,width-1070,height-580,imgsize-481831,overlay-economictimes/photo.jpg",
    //     "related": "AAPL",
    //     "source": "The Economic Times India",
    //     "summary": "NEW DELHI | CHENNAI: India may have to offer electronic manufacturers additional sops such as cheap credit and incentives for export along with infrastructure support in order to boost production and help the sector compete with China, Vietnam and Thailand, according to a top government official.These incentives, over and above the proposed reduction of corporate tax to 15% for new manufacturing units, are vital for India to successfully attract companies looking to relocate manufacturing facilities.“While the tax announcements made last week send a very good signal, in order to help attract investments, we will need additional initiatives,” the official told ET, pointing out that Indian electronic manufacturers incur 8-10% higher costs compared with other Asian countries.Sops that are similar to the incentives for export under the existing Merchandise Exports from India Scheme (MEIS) are what the industry requires, the person said.MEIS gives tax credit in the range of 2-5%. An interest subvention scheme for cheaper loans and a credit guarantee scheme for plant and machinery are some other possible measures that will help the industry, the official added.“This should be 2.0 (second) version of the electronic manufacturing cluster (EMC) scheme, which is aimed at creating an ecosystem with an anchor company plus its suppliers to operate in the same area,” he said.Last week, finance minister Nirmala Sitharaman announced a series of measures to boost economic growth including a scheme allowing any new manufacturing company incorporated on or after October 1, to pay income tax at 15% provided the company does not avail of any other exemption or incentives.",
    //     "url": "https://economictimes.indiatimes.com/industry/cons-products/electronics/more-sops-needed-to-boost-electronic-manufacturing-top-govt-official/articleshow/71321308.cms"
    //   },

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CompanyNews {
        pub category: String,
        pub datetime: i64,
        pub headline: String,
        pub id: i64,
        pub image: String,
        pub related: String,
        pub source: String,
        pub summary: String,
        pub url: String,
    }

    // Company Peers: Vec<String>

    // Basic Financials

    // {
    //    "series": {
    //     "annual": {
    //       "currentRatio": [
    //         {
    //           "period": "2019-09-28",
    //           "v": 1.5401
    //         },
    //         {
    //           "period": "2018-09-29",
    //           "v": 1.1329
    //         }
    //       ],
    //       "salesPerShare": [
    //         {
    //           "period": "2019-09-28",
    //           "v": 55.9645
    //         },
    //         {
    //           "period": "2018-09-29",
    //           "v": 53.1178
    //         }
    //       ],
    //       "netMargin": [
    //         {
    //           "period": "2019-09-28",
    //           "v": 0.2124
    //         },
    //         {
    //           "period": "2018-09-29",
    //           "v": 0.2241
    //         }
    //       ]
    //     }
    //   },
    //   "metric": {
    //     "10DayAverageTradingVolume": 32.50147,
    //     "52WeekHigh": 310.43,
    //     "52WeekLow": 149.22,
    //     "52WeekLowDate": "2019-01-14",
    //     "52WeekPriceReturnDaily": 101.96334,
    //     "beta": 1.2989,
    //   },
    //   "metricType": "all",
    //   "symbol": "AAPL"
    // }

    // Insider Sentiment

    // {
    //   "data":[
    //     {
    //       "symbol":"TSLA",
    //       "year":2021,
    //       "month":3,
    //       "change":5540,
    //       "mspr":12.209097
    //     },
    //     {
    //       "symbol":"TSLA",
    //       "year":2022,
    //       "month":1,
    //       "change":-1250,
    //       "mspr":-5.6179776
    //     },
    //     {
    //       "symbol":"TSLA",
    //       "year":2022,
    //       "month":2,
    //       "change":-1250,
    //       "mspr":-2.1459227
    //     },
    //     {
    //       "symbol":"TSLA",
    //       "year":2022,
    //       "month":3,
    //       "change":5870,
    //       "mspr":8.960191
    //     }
    //   ],
    //   "symbol":"TSLA"
    // }

    // Financials As Reported

    // {
    //   "cik": "320193",
    //   "data": [
    //     {
    //       "accessNumber": "0000320193-19-000119",
    //       "symbol": "AAPL",
    //       "cik": "320193",
    //       "year": 2019,
    //       "quarter": 0,
    //       "form": "10-K",
    //       "startDate": "2018-09-30 00:00:00",
    //       "endDate": "2019-09-28 00:00:00",
    //       "filedDate": "2019-10-31 00:00:00",
    //       "acceptedDate": "2019-10-30 18:12:36",
    //       "report": {
    //         "bs": {
    //           "Assets": 338516000000,
    //           "Liabilities": 248028000000,
    //           "InventoryNet": 4106000000,
    //           ...
    //         },
    //         "cf": {
    //           "NetIncomeLoss": 55256000000,
    //           "InterestPaidNet": 3423000000,
    //           ...
    //         },
    //         "ic": {
    //           "GrossProfit": 98392000000,
    //           "NetIncomeLoss": 55256000000,
    //           "OperatingExpenses": 34462000000,
    //            ...
    //         }
    //       }
    //     }
    //   ],
    //   "symbol": "AAPL"
    // }

    // SEC Filings

    // [
    //   {
    //     "accessNumber": "0001193125-20-050884",
    //     "symbol": "AAPL",
    //     "cik": "320193",
    //     "form": "8-K",
    //     "filedDate": "2020-02-27 00:00:00",
    //     "acceptedDate": "2020-02-27 06:14:21",
    //     "reportUrl": "https://www.sec.gov/ix?doc=/Archives/edgar/data/320193/000119312520050884/d865740d8k.htm",
    //     "filingUrl": "https://www.sec.gov/Archives/edgar/data/320193/000119312520050884/0001193125-20-050884-index.html"
    //   },
    //   {
    //     "accessNumber": "0001193125-20-039203",
    //     "symbol": "AAPL",
    //     "cik": "320193",
    //     "form": "8-K",
    //     "filedDate": "2020-02-18 00:00:00",
    //     "acceptedDate": "2020-02-18 06:24:57",
    //     "reportUrl": "https://www.sec.gov/ix?doc=/Archives/edgar/data/320193/000119312520039203/d845033d8k.htm",
    //     "filingUrl": "https://www.sec.gov/Archives/edgar/data/320193/000119312520039203/0001193125-20-039203-index.html"
    //   },
    //   ...
    // ]

    // Recommendation Trends

    // [
    //   {
    //     "buy": 24,
    //     "hold": 7,
    //     "period": "2020-03-01",
    //     "sell": 0,
    //     "strongBuy": 13,
    //     "strongSell": 0,
    //     "symbol": "AAPL"
    //   },
    //   {
    //     "buy": 17,
    //     "hold": 13,
    //     "period": "2020-02-01",
    //     "sell": 5,
    //     "strongBuy": 13,
    //     "strongSell": 0,
    //     "symbol": "AAPL"
    //   }
    // ]

    // Earnings Calendar

    // {
    //   "earningsCalendar": [
    //     {
    //       "date": "2020-01-28",
    //       "epsActual": 4.99,
    //       "epsEstimate": 4.5474,
    //       "hour": "amc",
    //       "quarter": 1,
    //       "revenueActual": 91819000000,
    //       "revenueEstimate": 88496400810,
    //       "symbol": "AAPL",
    //       "year": 2020
    //     },
    //     {
    //       "date": "2019-10-30",
    //       "epsActual": 3.03,
    //       "epsEstimate": 2.8393,
    //       "hour": "amc",
    //       "quarter": 4,
    //       "revenueActual": 64040000000,
    //       "revenueEstimate": 62985161760,
    //       "symbol": "AAPL",
    //       "year": 2019
    //     }
    //    ]
    // }

    // Quote

    // {
    //   "c": 261.74,
    //   "h": 263.31,
    //   "l": 260.68,
    //   "o": 261.07,
    //   "pc": 259.45,
    //   "t": 1582641000
    // }

    #[derive(Serialize, Deserialize)]
    pub struct Quote {
        c: f64,
        h: f64,
        l: f64,
        o: f64,
        pc: f64,
        t: i64,
    }

    // Candlestick Data

    // {
    //   "c": [
    //     217.68,
    //     221.03,
    //     219.89
    //   ],
    //   "h": [
    //     222.49,
    //     221.5,
    //     220.94
    //   ],
    //   "l": [
    //     217.19,
    //     217.1402,
    //     218.83
    //   ],
    //   "o": [
    //     221.03,
    //     218.55,
    //     220
    //   ],
    //   "s": "ok",
    //   "t": [
    //     1569297600,
    //     1569384000,
    //     1569470400
    //   ],
    //   "v": [
    //     33463820,
    //     24018876,
    //     20730608
    //   ]
    // }

    // }
}

// #[cfg(test)]
// mod tests {
//     use crate::{app::State, SEARCH_STRING};

//     #[test]
//     fn stock_symbol_init_works() {
//         let mut state = State::default();
//         let stock_symbols = state.stock_symbols_init();
//         assert!(stock_symbols.is_ok());
//     }

//     #[test]
//     fn bad_market_input_gives_error() {
//         let mut state = State::default();
//         *SEARCH_STRING.inner() = "bad market".to_string();
//         let res = state.choose_market();
//         assert_eq!(res, "No market called bad market exists");
//     }
// }
