use color_eyre::eyre::Result as RepResult;

use crate::{
    binaries::{init_search_path, is_binary_exist},
    cli::{AppMode, Cli},
    simple_app::SimpleApp,
    interactive_app::InteractiveApp,
};

pub trait App {
    fn with_args(args: Cli) -> Self;
    fn run(&mut self) -> RepResult<()>;
}

pub fn run_app(mut args: Cli) -> RepResult<()> {
    let app_mode = AppMode::from(&(&args).app_mode);

    if let Some(path) = &args.search_path {
        init_search_path(&path)
    }

    if args.show_descriptions
    && (!is_binary_exist("man") || !is_binary_exist("groff"))
    {
        args.show_descriptions = false;
    }

    match app_mode {
        AppMode::Simple => SimpleApp::with_args(args).run(),
        AppMode::Interactive => InteractiveApp::with_args(args).run(),
    }
}
