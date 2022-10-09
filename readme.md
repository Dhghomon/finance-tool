Just a tool to get financial data and display it somehow, probably using tui. So far after the first broadcast it uses crossterm to get keyboard input and has one method to make blocking requests via reqwest through FinnHub's free API.

Todo: 

- Allow API choice on top left block via arrow keys - partially done (left + right keys work) Add up and down later
- Find company automatically instead of having to remember symbol and moving to company profile to manually type in
- Now has default info if market doesn't change so probably don't need to bring in default info for a bunch of markets. todo: think about *maybe* a scoped thread for this call because it takes a long time
- Turn that market symbols function (the one that gets all the company symbols for a single market) back into one that returns a Result, probably split into two (one that checks to see if the input is valid, then another with a Result depending on what comes back from FinnHub)
- Show which market is chosen (e.g. F = Frankfurt? US is obvious but other ones not so much.)

API stuff todo:

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