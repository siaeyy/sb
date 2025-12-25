#[cfg(not(unix))]
compile_error!("No Win");

mod clipboard;
mod cli;
mod man;
mod roff;
mod binaries;
mod descriptions;
mod widgets;
mod states;
mod app;
mod simple_app;
mod interactive_app;

use clap::Parser;
use color_eyre::eyre::Result as RepResult;

#[cfg(target_os = "linux")]
use clipboard::handle_clipboard_request;
use cli::Cli;

use app::run_app;

fn main() -> RepResult<()> {
    #[cfg(target_os = "linux")]
    let _ = handle_clipboard_request();

    run_app(Cli::parse())
}
