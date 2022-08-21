pub const API_KEY: &str = include_str!("..\\key.txt");

// pub const EXCHANGE_CODES: [&str; 10] = [
//   "  AS",
//   "AT",
//   "AX",
//   "BA",
//   "BC",
//   "BD",
//   "BE",
//   "BK",
//   "BO",
//   "BR",
//   "CA",
//   "CN",
//   "CO",
//   "CR",
//   "DB",
//   "DE",
//   "DU",
//   "F",
//   "HE",
//   "HK",
//   "HM",
//   "IC",
//   "IR",
//   "IS",
//   "JK",
//   "JO",
//   "KL",
//   "KQ",
//   "KS",
//   "L",
//   "LN",
//   "LS",
//   "MC",
//   "ME",
//   "MI",
//   "MU",
//   "MX",
//   "NE",
//   "NL",
//   "NS",
//   "NZ",
//   "OL",
//   "PA",
//   "PM",
//   "PR",
//   "QA",
//   "RG",
//   "SA",
//   "SG",
//   "SI",
//   "SN",
//   "SR",
//   "SS",
//   "ST",
//   "SW",
//   "SZ",
//   "T",
//   "TA",
//   "TL",
//   "TO",
//   "TW",
//   "TWO",
//   "US",
//   "V",
//   "VI",
//   "VN",
//   "VS",
//   "WA",
//   "HA",
//   "SX",
//   "TG",
//   "SC"
// ];

pub mod app {
    use std::{fmt::Display, str::FromStr};

    use anyhow::{anyhow, Context, Error};
    use reqwest::blocking::Client;
    use serde::de::DeserializeOwned;
    use tui::{
        style::{Color, Modifier, Style},
        text::Span,
    };

    use crate::{api::CompanyProfile, API_KEY};

    pub struct FinanceClient {
        pub url: String,
        pub client: Client,
        pub search_string: String,   // push + pop // MSFT
        pub current_content: String, // Results etc. of searches
        pub choice: ApiChoice,
    }

    impl FinanceClient {
        pub fn finnhub_request<T: DeserializeOwned + Display>(
            &self,
            url: String,
        ) -> Result<String, Error> {
            let response = self
                .client
                .get(url)
                .header("X-Finnhub-Token", API_KEY)
                .send()
                .with_context(|| "Couldn't send via client")?;
            let text = response.text().with_context(|| "No text for some reason")?;
            let finnhub_reply: T = serde_json::from_str(&text).with_context(|| {
                format!(
                    "Couldn't deserialize {} into CompanyProfile struct.\nText from Finnhub: '{text}'",
                    self.search_string
                )
            })?;
            Ok(finnhub_reply.to_string())
        }

        pub fn company_profile(&self) -> Result<String, Error> {
            let url = format!("{}/stock/profile2?symbol={}", self.url, self.search_string);
            let company_info = self.finnhub_request::<CompanyProfile>(url)?;
            Ok(company_info)
        }

        /// /stock/symbol?exchange=US
        pub fn stock_symbol(&self) -> Result<String, Error> {
            todo!()
        }

        /// company-news?symbol=AAPL&from=2021-09-01&to=2021-09-09
        /// Required: date + symbol
        pub fn company_news(&self) -> Result<String, Error> {
            todo!()
        }

        /// news?category=general
        /// This parameter can be 1 of the following values general, forex, crypto, merger
        pub fn market_news(&self) -> Result<String, Error> {
            todo!()
        }

        pub fn switch(&mut self) {
            use ApiChoice::*;
            self.choice = match self.choice {
                SymbolSearch => CompanyProfile,
                CompanyProfile => StockSymbol,
                StockSymbol => MarketNews,
                MarketNews => CompanyNews,
                CompanyNews => SymbolSearch,
            }
        }
        pub fn all_choices(&self) -> Vec<Span> {
            use ApiChoice::*; // SymbolSearch
            let choices = [
                SymbolSearch,
                CompanyProfile,
                StockSymbol,
                MarketNews,
                CompanyNews,
            ];
            let choices = choices.into_iter().map(|choice| choice.to_string());

            let mut even_odd = [true, false].into_iter().cycle();
            choices
                .into_iter()
                .map(|choice_string| {
                    let black = even_odd.next().unwrap();
                    let bg = if black { Color::Black } else { Color::DarkGray };
                    let current_choice = format!("{}", self.choice);
                    if choice_string == current_choice {
                        Span::styled(
                            format!(" {choice_string} "),
                            Style::default()
                                .fg(Color::LightYellow)
                                .bg(bg)
                                .add_modifier(Modifier::UNDERLINED),
                        )
                    } else {
                        Span::styled(
                            format!(" {choice_string} "),
                            Style::default().fg(Color::White).bg(bg),
                        )
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
    }

    impl std::fmt::Display for ApiChoice {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use ApiChoice::*;
            let output = match self {
                SymbolSearch => "Company symbol",
                CompanyProfile => "Company info",
                StockSymbol => "Stock symbol",
                MarketNews => "Market news",
                CompanyNews => "Company news",
            };
            write!(f, "{}", output)
        }
    }

    // Todo!() probably delete this because it feels like overkill
    #[derive(Debug, Display, EnumIter, PartialEq, Eq)]
    pub enum ExchangeCodes {
        AS,
        AT,
        AX,
        BA,
        BC,
        BD,
        BE,
        BK,
        BO,
        BR,
        CA,
        CN,
        CO,
        CR,
        DB,
        DE,
        DU,
        F,
        HE,
        HK,
        HM,
        IC,
        IR,
        IS,
        JK,
        JO,
        KL,
        KQ,
        KS,
        L,
        LN,
        LS,
        MC,
        ME,
        MI,
        MU,
        MX,
        NE,
        NL,
        NS,
        NZ,
        OL,
        PA,
        PM,
        PR,
        QA,
        RG,
        SA,
        SG,
        SI,
        SN,
        SR,
        SS,
        ST,
        SW,
        SZ,
        T,
        TA,
        TL,
        TO,
        TW,
        TWO,
        US,
        V,
        VI,
        VN,
        VS,
        WA,
        HA,
        SX,
        TG,
        SC,
    }

    use strum::IntoEnumIterator;
    use strum_macros::{Display, EnumIter};

    impl FromStr for ExchangeCodes {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            ExchangeCodes::iter()
                .find(|code| &code.to_string() == s)
                .ok_or_else(|| anyhow!("Couldn't get ExchangeCode from {s}"))
        }
    }
}

/// Structs and enums for the Finnhub API.
pub mod api {

    /// Serialize = into JSON
    ///
    /// Deserialize = into Rust type
    #[derive(Debug, Serialize, Deserialize)]
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

    /// description": "APPLE INC",
    /// "displaySymbol": "AAPL",
    ///   "symbol": "AAPL",
    ///   "type": "Common Stock"
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize)]
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
    #[derive(Serialize, Deserialize)]
    struct StockSymbol {
        currency: String,
        description: String,
        #[serde(rename = "displaySymbol")]
        display_symbol: String,
        figi: String,
        mic: String,
        symbol: String,
        #[serde(rename = "type")]
        type_: String,
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
    #[derive(Serialize, Deserialize)]
    struct MarketNews {
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

    #[derive(Serialize, Deserialize)]
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
