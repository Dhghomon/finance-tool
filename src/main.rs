use std::sync::mpsc::sync_channel;

use finance_tool::app::{handle_event, FinanceClient, State};
use tui::{backend::CrosstermBackend, Terminal};

// const COMPANY_STR: &str = include_str!("../company_symbols.json");

fn main() {
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // (SyncSender<Command>, Receiver<Command>)
    let (command_sender, command_receiver) = sync_channel(2);
    // (SyncSender<ApiCommand>, Receiver<ApiCommand>)
    let (api_sender, api_receiver) = sync_channel(2);

    let mut state = State::new(api_sender, command_receiver);
    let cloned = command_sender.clone();
    let finance_client = FinanceClient::new(cloned, api_receiver);

    state.stock_symbols_init().unwrap();
    terminal.clear().unwrap();
    state.draw_terminal(&mut terminal);

    std::thread::spawn(move || loop {
        finance_client.receive_command();
    });

    std::thread::spawn(move || loop {
        state.receive_command();
        state.check_self();
        state.draw_terminal(&mut terminal);
    });

    loop {
        handle_event(&command_sender);
    }
}
