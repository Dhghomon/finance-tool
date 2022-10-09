use std::{
    fmt::Debug,
    sync::{Mutex, MutexGuard},
};

pub const API_KEY: &str = include_str!("..\\key.txt");

pub const EXCHANGE_CODES: [&str; 72] = [
    "AS", "AT", "AX", "BA", "BC", "BD", "BE", "BK", "BO", "BR", "CA", "CN", "CO", "CR", "DB", "DE",
    "DU", "F", "HE", "HK", "HM", "IC", "IR", "IS", "JK", "JO", "KL", "KQ", "KS", "L", "LN", "LS",
    "MC", "ME", "MI", "MU", "MX", "NE", "NL", "NS", "NZ", "OL", "PA", "PM", "PR", "QA", "RG", "SA",
    "SG", "SI", "SN", "SR", "SS", "ST", "SW", "SZ", "T", "TA", "TL", "TO", "TW", "TWO", "US", "V",
    "VI", "VN", "VS", "WA", "HA", "SX", "TG", "SC",
];

pub const FINNHUB_URL: &str = "https://finnhub.io/api/v1";

#[derive(PartialEq, Eq, Debug)]
pub enum Window {
    ApiChoice,
    Results,
}

pub struct GlobalString(Mutex<String>);

impl GlobalString {
    pub const fn new() -> Self {
        Self(Mutex::new(String::new()))
    }
    pub fn inner(&self) -> MutexGuard<'_, std::string::String> {
        self.0.lock().unwrap()
    }
}

pub fn debug(message: impl Debug) {
    match std::env::args().nth(1) {
        Some(arg) if &arg == "debug" => {
            let message_string = format!("{message:?}");
            *CURRENT_CONTENT.inner() = message_string;
        }
        _ => {}
    }
}

pub static SEARCH_STRING: GlobalString = GlobalString(Mutex::new(String::new()));
pub static CURRENT_CONTENT: GlobalString = GlobalString(Mutex::new(String::new()));

pub mod app {
    use std::{
        fmt::Debug,
        fs::File,
        io::{Read, Write},
    };

    use anyhow::{Context, Error};
    use crossterm::event::{read, Event, KeyCode, KeyEvent};
    use reqwest::{
        blocking::Client,
        header::{HeaderMap, HeaderValue},
        ClientBuilder,
    };
    use serde::de::DeserializeOwned;
    use tui::{
        style::{Color, Style},
        text::Span,
    };

    use crate::{
        api::{CompanyProfile, MarketNews, StockSymbol},
        debug, Window, API_KEY, CURRENT_CONTENT, EXCHANGE_CODES, FINNHUB_URL, SEARCH_STRING,
    };

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
            CURRENT_CONTENT.inner().clear();
            self.current_index = match self.current_index.checked_sub(1) {
                Some(okay_number) => okay_number,
                None => self.all_apis.len() - 1,
            };
        }
        pub fn right(&mut self) {
            CURRENT_CONTENT.inner().clear();
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
        pub current_window: Window,
        pub api_choices: TotalApiChoices,
        pub current_market: String,
        pub companies: Vec<String>,
    }

    #[derive(Debug)]
    pub struct State {
        pub current_window: Window,
        pub api_choices: TotalApiChoices,
        pub current_market: String,
        pub companies: Vec<String>,
    }

    impl Default for State {
        fn default() -> Self {
            Self {
                current_window: Window::ApiChoice,
                api_choices: TotalApiChoices::default(),
                current_market: "US".to_string(),
                companies: Vec::new(),
            }
        }
    }

    // impl State {
    //     pub fn handle_event(&mut self) {
    //         match read().unwrap() {
    //             Event::Key(key_event) => {
    //                 let KeyEvent {
    //                     code, modifiers, ..
    //                 } = key_event;
    //                 // Typing event
    //                 match (code, modifiers) {
    //                     (KeyCode::Char(c), _) => {
    //                         SEARCH_STRING.inner().push(c);
    //                     }
    //                     (KeyCode::Esc, _) => {
    //                         SEARCH_STRING.inner().clear();
    //                     }
    //                     (KeyCode::Backspace, _) => {
    //                         SEARCH_STRING.inner().pop();
    //                     }
    //                     (KeyCode::Enter, _) => match self.api_choice() {
    //                         ApiChoice::CompanyProfile => {
    //                             *CURRENT_CONTENT.inner() = match self.company_profile() {
    //                                 Ok(search_result) => search_result,
    //                                 Err(e) => e.to_string(),
    //                             }
    //                         }
    //                         ApiChoice::GetMarket => {
    //                             *CURRENT_CONTENT.inner() = self.choose_market();
    //                         }
    //                         ApiChoice::MarketNews => {
    //                             let string_output = self.market_news().unwrap();
    //                             *CURRENT_CONTENT.inner() = string_output;
    //                         }
    //                         _ => {}
    //                     },
    //                     (KeyCode::Left, _) => {
    //                         if self.current_window == Window::ApiChoice {
    //                             self.api_choices.left();
    //                         }
    //                     }
    //                     (KeyCode::Right, _) => {
    //                         if self.current_window == Window::ApiChoice {
    //                             self.api_choices.right();
    //                         }
    //                     }
    //                     (KeyCode::Tab, _) => {
    //                         self.switch_window();
    //                     }
    //                     (_, _) => {}
    //                 }
    //             }
    //             Event::Mouse(_) => {}
    //             Event::Resize(num1, num2) => {
    //                 println!("Window has been resized to {num1}, {num2}");
    //             }
    //             Event::Paste(_s) => {}
    //             _ => {}
    //         }
    //         if self.api_choice() == ApiChoice::SymbolSearch && !SEARCH_STRING.inner().is_empty() {
    //             *CURRENT_CONTENT.inner() = self.company_search(&SEARCH_STRING.inner());
    //         }
    //     }

    //     pub fn stock_symbols_init(&mut self) -> Result<(), Error> {
    //         match File::open("company_symbols.txt") {
    //             Ok(mut file) => {
    //                 let mut company_string = String::new();
    //                 file.read_to_string(&mut company_string)?;
    //                 let all_companies = company_string
    //                     .lines()
    //                     .map(|line| line.to_string())
    //                     .collect::<Vec<String>>();
    //                 self.companies = all_companies;
    //                 Ok(())
    //             }
    //             Err(_) => {
    //                 let company_info = self
    //                     .stock_symbols()?
    //                     .into_iter()
    //                     .map(|s| format!("{}: {}\n", s.description, s.display_symbol))
    //                     .collect::<Vec<String>>();
    //                 self.companies = company_info.clone();

    //                 let mut file = File::create("company_symbols.txt")?;
    //                 let num = self
    //                     .companies
    //                     .iter()
    //                     .fold(0, |first, second| second.len() + first);
    //                 let mut output_string = String::with_capacity(num);
    //                 company_info.iter().for_each(|s| {
    //                     output_string.push_str(s);
    //                     output_string.push('\n');
    //                 });
    //                 file.write_all(output_string.as_bytes())?;
    //                 Ok(())
    //             }
    //         }
    //     }

    //     /// User hits enter, checks to see if market exists, if not, stay with original one
    //     pub fn choose_market(&mut self) -> String {
    //         debug(format!("Choosing market: {self:?}"));

    //         if self.current_market == *SEARCH_STRING.inner() {
    //             return format!("Already using market {}", SEARCH_STRING.inner());
    //         }
    //         match EXCHANGE_CODES
    //             .iter()
    //             .find(|code| **code == *SEARCH_STRING.inner())
    //         {
    //             // e.g. user types "T", which is valid
    //             Some(good_market_code) => {
    //                 // todo! take this unwrap back out
    //                 // Add debugging window or something
    //                 let stock_symbols = self.stock_symbols().unwrap();
    //                 // Now self.current_market is "T"
    //                 self.current_market = good_market_code.to_string();
    //                 SEARCH_STRING.inner().clear();
    //                 self.companies = stock_symbols
    //                     .into_iter()
    //                     .map(|info| format!("{}: {}", info.description, info.display_symbol))
    //                     .collect::<Vec<_>>();
    //                 format!(
    //                     "Successfully got company info from market {}",
    //                     self.current_market
    //                 )
    //             }
    //             // user types something that isn't a market
    //             None => format!("No market called {} exists", SEARCH_STRING.inner()),
    //         }
    //     }

    //     pub fn switch_window(&mut self) {
    //         self.current_window = match self.current_window {
    //             Window::ApiChoice => Window::Results,
    //             Window::Results => Window::ApiChoice,
    //         }
    //     }

    //     // todo!() turn this into Tables: 3*3 and then later 4*4
    //     pub fn all_choices(&self) -> Vec<Span> {
    //         let choices = &self.api_choices.all_apis;

    //         choices
    //             .iter()
    //             .enumerate()
    //             .map(|(index, api_name)| {
    //                 if self.api_choices.current_index == index {
    //                     Span::styled(format!("{api_name}"), Style::default().bg(Color::Gray))
    //                 } else {
    //                     Span::styled(format!("{api_name}"), Style::default().bg(Color::Black))
    //                 }
    //             })
    //             .collect::<Vec<_>>()
    //     }
    // }

    impl Default for FinanceClient {
        fn default() -> Self {
            let mut headers = HeaderMap::new();
            headers.insert("X-Finnhub-Token", HeaderValue::from_static(API_KEY));
            let client = Client::builder().default_headers(headers).build().unwrap();

            FinanceClient {
                client,
                current_window: Window::ApiChoice,
                api_choices: TotalApiChoices::default(),
                current_market: "US".to_string(),
                companies: Vec::new(),
            }
        }
    }

    /// Vec<StockSymbol>
    impl FinanceClient {
        pub fn api_choice(&self) -> ApiChoice {
            self.api_choices.current_api()
        }

        pub fn handle_event(&mut self) {
            match read().unwrap() {
                Event::Key(key_event) => {
                    let KeyEvent {
                        code, modifiers, ..
                    } = key_event;
                    // Typing event
                    match (code, modifiers) {
                        (KeyCode::Char(c), _) => {
                            SEARCH_STRING.inner().push(c);
                        }
                        (KeyCode::Esc, _) => {
                            SEARCH_STRING.inner().clear();
                        }
                        (KeyCode::Backspace, _) => {
                            SEARCH_STRING.inner().pop();
                        }
                        (KeyCode::Enter, _) => match self.api_choice() {
                            ApiChoice::CompanyProfile => {
                                *CURRENT_CONTENT.inner() = match self.company_profile() {
                                    Ok(search_result) => search_result,
                                    Err(e) => e.to_string(),
                                }
                            }
                            ApiChoice::GetMarket => {
                                *CURRENT_CONTENT.inner() = self.choose_market();
                            }
                            ApiChoice::MarketNews => {
                                let string_output = self.market_news().unwrap();
                                *CURRENT_CONTENT.inner() = string_output;
                            }
                            _ => {}
                        },
                        (KeyCode::Left, _) => {
                            if self.current_window == Window::ApiChoice {
                                self.api_choices.left();
                            }
                        }
                        (KeyCode::Right, _) => {
                            if self.current_window == Window::ApiChoice {
                                self.api_choices.right();
                            }
                        }
                        (KeyCode::Tab, _) => {
                            self.switch_window();
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
            if self.api_choice() == ApiChoice::SymbolSearch && !SEARCH_STRING.inner().is_empty() {
                *CURRENT_CONTENT.inner() = self.company_search(&SEARCH_STRING.inner());
            }
        }

        pub fn single_request<T: DeserializeOwned + Debug>(&self, url: String) -> Result<T, Error> {
            let text = self.get_text(url)?;
            let finnhub_reply: T = serde_json::from_str(&text).with_context(|| {
                format!(
                    "Couldn't deserialize {} into CompanyProfile struct.\nText from Finnhub: '{text}'",
                    SEARCH_STRING.inner()
                )
            })?;
            Ok(finnhub_reply)
        }

        pub fn multi_request<T: DeserializeOwned + Debug>(
            &self,
            url: String,
        ) -> Result<Vec<T>, Error> {
            let text = self.get_text(url)?;

            let finnhub_reply: Vec<T> = serde_json::from_str(&text).with_context(|| {
                format!(
                    "Couldn't deserialize {} into CompanyProfile struct.\nText from Finnhub: '{text}'",
                    SEARCH_STRING.inner()
                )
            })?;
            Ok(finnhub_reply)
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

        pub fn company_profile(&self) -> Result<String, Error> {
            // /stock/profile?symbol=AAPL
            let url = format!(
                "{FINNHUB_URL}/stock/profile2?symbol={}",
                SEARCH_STRING.inner()
            );
            let company_info = self.single_request::<CompanyProfile>(url)?;
            SEARCH_STRING.inner().clear();
            Ok(company_info.to_string())
        }

        pub fn stock_symbols_init(&mut self) -> Result<(), Error> {
            match File::open("company_symbols.txt") {
                Ok(mut file) => {
                    let mut company_string = String::new();
                    file.read_to_string(&mut company_string)?;
                    let all_companies = company_string
                        .lines()
                        .map(|line| line.to_string())
                        .collect::<Vec<String>>();
                    self.companies = all_companies;
                    Ok(())
                }
                Err(_) => {
                    let company_info = self
                        .stock_symbols()?
                        .into_iter()
                        .map(|s| format!("{}: {}\n", s.description, s.display_symbol))
                        .collect::<Vec<String>>();
                    self.companies = company_info.clone();

                    let mut file = File::create("company_symbols.txt")?;
                    let num = self
                        .companies
                        .iter()
                        .fold(0, |first, second| second.len() + first);
                    let mut output_string = String::with_capacity(num);
                    company_info.iter().for_each(|s| {
                        output_string.push_str(s);
                        output_string.push('\n');
                    });
                    file.write_all(output_string.as_bytes())?;
                    Ok(())
                }
            }
        }

        pub fn get_text(&self, url: String) -> Result<String, Error> {
            let url = format!("{url}&token={API_KEY}");
            println!("{url}");
            self.client
                .get(url)
                .header("X-Finnhub-Token", API_KEY)
                .send()
                .with_context(|| "Couldn't send via client")?
                .text()
                .with_context(|| "No text for some reason")
        }

        /// /stock/symbol?exchange=US
        pub fn stock_symbols(&self) -> Result<Vec<StockSymbol>, Error> {
            let url = format!(
                "{FINNHUB_URL}/stock/symbol?exchange={}",
                self.current_market
            );

            let text = self.get_text(url)?;
            let mut new_file = File::create("stock_symbols.json")?;
            write!(&mut new_file, "{}", text)?;
            let stock_symbols: Vec<StockSymbol> = serde_json::from_str(&text).unwrap();
            Ok(stock_symbols)
        }

        /// User hits enter, checks to see if market exists, if not, stay with original one
        pub fn choose_market(&mut self) -> String {
            debug(format!("Choosing market: {self:?}"));

            if self.current_market == *SEARCH_STRING.inner() {
                return format!("Already using market {}", SEARCH_STRING.inner());
            }
            match EXCHANGE_CODES
                .iter()
                .find(|code| **code == *SEARCH_STRING.inner())
            {
                // e.g. user types "T", which is valid
                Some(good_market_code) => {
                    // todo! take this unwrap back out
                    // Add debugging window or something
                    let stock_symbols = self.stock_symbols().unwrap();
                    // Now self.current_market is "T"
                    self.current_market = good_market_code.to_string();
                    SEARCH_STRING.inner().clear();
                    self.companies = stock_symbols
                        .into_iter()
                        .map(|info| format!("{}: {}", info.description, info.display_symbol))
                        .collect::<Vec<_>>();
                    format!(
                        "Successfully got company info from market {}",
                        self.current_market
                    )
                }
                // user types something that isn't a market
                None => format!("No market called {} exists", SEARCH_STRING.inner()),
            }
        }

        /// company-news?symbol=AAPL&from=2021-09-01&to=2021-09-09
        /// Required: date + symbol
        pub fn company_news(&self) -> Result<String, Error> {
            todo!()
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

        pub fn switch_window(&mut self) {
            self.current_window = match self.current_window {
                Window::ApiChoice => Window::Results,
                Window::Results => Window::ApiChoice,
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

    use std::fmt::Display;

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
    #[derive(Debug, Serialize, Deserialize)]
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
    struct CompanyNews {
        category: String,
        datetime: i64,
        headline: String,
        id: i64,
        image: String,
        related: String,
        source: String,
        summary: String,
        url: String,
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

#[cfg(test)]
mod tests {
    use crate::{app::FinanceClient, SEARCH_STRING};

    #[test]
    fn stock_symbol_init_works() {
        let mut client = FinanceClient::default();
        let stock_symbols = client.stock_symbols_init();
        assert!(stock_symbols.is_ok());
    }

    #[test]
    fn bad_market_input_gives_error() {
        let mut client = FinanceClient::default();
        *SEARCH_STRING.inner() = "bad market".to_string();
        let res = client.choose_market();
        assert_eq!(res, "No market called bad market exists");
    }
}
