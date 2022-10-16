use std::{io::Stdout, sync::mpsc::sync_channel};

use finance_tool::{
    app::{FinanceClient, State, handle_event2},
    Window,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
    Terminal,
};

// Company news
// Small window for error / debug messages

/// Select Market
///
enum Market {}

/// todo! Make into real error
enum ClientError {
    IncorrectInput,
}

// const COMPANY_STR: &str = include_str!("../company_symbols.json");

//    loop -> Right -> State (in its own scoped thread receives RightClick StateCommand)
//      RightClick -> simple change state
//      ApiCall -> ApiClient (in its own scoped thread)
//

fn main() {
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // {
    //     {
    //         // State Client
    //         // .recv() -- from User
    //         // .draw()
    //         // .recv() -- from ApiClient
    //     }
    //     {
    //         // ApiClient .recv()
    //         // receives api call order
    //         // sends to State Client
    //     }

    //     loop {
    //         // blocking function
    //         // terminal input
    //     }
    // }

    //let mut shutdown_requested = false;

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

    std::thread::spawn(move || {
        loop {
            finance_client.receive_command();
        }
    });

    std::thread::spawn(move || {
        loop {
            state.receive_command();
            state.draw_terminal(&mut terminal);
        }
    });

    loop {
        handle_event2(&command_sender);
    }

    // std::thread::scope(|s| {
    //     // State loops around and does stuff
    //     s.spawn(|| {
    //         while !shutdown_requested {
    //             state.receive_command();
    //             state.draw_terminal(&mut terminal);
    //         }
    //     });
    //     // Finance Client loops around and does stuff
    //     // s.spawn(|| {
    //     //     while !shutdown_requested {
    //     //         finance_client.receive_command();
    //     //     }
    //     // });


    // });

    // loop {
    //     // Handles key events and decides what to do
    //     state.handle_event();
    //     terminal.clear()?;
    //     draw_terminal(&mut terminal, &state);
    // }
}
