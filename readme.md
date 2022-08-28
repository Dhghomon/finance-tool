Just a tool to get financial data and display it somehow, probably using tui. So far after the first broadcast it uses crossterm to get keyboard input and has one method to make blocking requests via reqwest through FinnHub's free API.

Todo: 

- Move API choices from top of block 1 to table: use all arrow keys to choose
- Find company automatically instead of having to remember symbol and moving to company profile to manually type in
- Think about getting info for all 72 exchanges one time because request takes a lot of time / do some async because the user probably won't start searching until a few seconds have passed (getting all company symbols seems to take a few seconds) - or do it with a thread? Could be interesting to try both
- Turn that market symbols function (the one that gets all the company symbols for a single market) back into one that returns a Result, probably split into two (one that checks to see if the input is valid, then another with a Result depending on what comes back from FinnHub)

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