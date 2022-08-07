Just a tool to get financial data and display it somehow, probably using tui. So far after the first broadcast it uses crossterm to get keyboard input and has one method to make blocking requests via reqwest through FinnHub's free API.

Todo: 

- Think about getting rid of all the Mutexes, they are kind of annoying
- Taking user input and then match inside the tui draw function
- Error handling

API stuff todo:

stock symbols
company profile 2 - ISIN, company symbol - Microsoft치고, MSFT가 나오고, MSFT로 검색하기
market news
company news
peers
basic financials
Insider transactions?
Insider sentiment
Financials as reported
SEC filings
IPO calendar
EPS Surprises
Earnings calendar
Stock quote
Candles?
Forex exchanges
forex symbols
USPTO patents
Visa application
Senate lobbying
USA spending
COVID-19 by state
Country list