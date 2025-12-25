use clap::{Args, Parser};

#[derive(Default, Parser, Debug)]
#[command(
    version, about,
    long_about = None,
    next_line_help = true,
    disable_help_flag = true,
    disable_version_flag = true,
)]
pub struct Cli {
    /// Print help.
    #[arg(
        short = 'h',
        long = "help",
        action = clap::ArgAction::Help,
    )]
    _help: (),

    /// Print version.
    #[arg(
        short = 'v',
        long = "version",
        action = clap::ArgAction::Version,
    )]
    _version: (),
    
    #[command(flatten)]
    pub app_mode: AppModeArg,

    /// Copy the first binary name
    /// in the search result to the clipboard.
    /// Input must be given!
    #[arg(
        short = 'c',
        long = "copy",
        requires = "search_input",
        verbatim_doc_comment,
    )]
    pub should_copy_result: bool,

    /// Show the descriptions of binaries in the search result.
    /// "man-db" and "groff" are required!
    #[arg(
        short = 'd',
        long = "descriptions",
        verbatim_doc_comment,
    )]
    pub show_descriptions: bool,

    /// Set the length of binary names to display in the result.
    /// Simple app mode must be enabled!
    #[arg(
        short = 'l',
        long = "length",
        requires = "simple_ui_mode",
        default_value_t = 10,
        verbatim_doc_comment,
    )]
    pub result_length: usize,

    /// Set the path variable for searching binaries in it.
    /// Default value depends on $PATH environment variable.
    #[arg(
        short = 'p',
        long = "path",
        verbatim_doc_comment,
    )]
    pub search_path: Option<String>,

    /// Name input for searching binaries with similar name.
    #[arg(
        group = "search_input",
        value_name = "SEARCH_INPUT",
    )]
    pub input: Option<String>,
}

#[derive(Default, Args, Debug)]
#[group(multiple = false)]
pub struct AppModeArg {
    /// Set the app mode to simple,
    /// just show result of the given input.
    #[arg(
        short = 's',
        long,
        group = "simple_ui_mode",
        requires = "search_input",
        verbatim_doc_comment,
    )]
    pub simple: bool,

    /// Set the app mode to interactive,
    /// change the input and see the result interactively.
    #[arg(
        short = 'i',
        long,
        default_value_t = true,
        verbatim_doc_comment,
    )]
    pub interactive: bool,
}

pub enum AppMode {
    Simple,
    Interactive,
}

impl From<&AppModeArg> for AppMode {
    fn from(value: &AppModeArg) -> Self {
        match (value.simple, value.interactive) {
            (true, _) => AppMode::Simple,
            _ => AppMode::Interactive,
        }
    }
}
