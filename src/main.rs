use clap::{ColorChoice, Parser, ValueEnum};
use env_logger;
use log;
use owo_colors::{OwoColorize, Stream::Stdout};

/// Find SEARCH_TERM in available nix packages and sort results by relevance
///
/// List up to three columns, the latter two being optional:
/// PACKAGE_NAME  [PACKAGE_VERSION]  [PACKAGE_DESCRIPTION]
///
/// Matches are sorted by type. Show 'exact' matches first, then 'direct' matches, and finally 'indirect' matches.
///
///   exact     SEARCH_TERM
///   direct    SEARCH_TERM-bar
///   indirect  foo-SEARCH_TERM-bar (or match other columns)
#[derive(Parser)]
#[command(author, version, verbatim_doc_comment)]
struct Cli {
    /// Highlight search matches in color
    #[arg(short, long="color", visible_alias="colour", default_value_t = ColorChoice::Auto)]
    color: ColorChoice,

    /// Choose columns to show
    #[arg(short='C', long="columns", default_value_t = Columns::All, value_enum)]
    columns: Columns,

    /// Turn debugging information on
    ///
    /// Use multiple times for increased verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Flip the order of matches and sorting
    #[arg(short, long, default_value_t = false, default_missing_value = "true", num_args=0..=1, action = clap::ArgAction::Set)]
    flip: bool,

    /// Refresh package cache and exit
    #[arg(short, long)]
    refresh: bool,

    /// Separate match types with a newline
    #[arg(short, long, default_value_t = false, default_missing_value = "true", num_args=0..=1, action = clap::ArgAction::Set)]
    separate: bool,

    /// Show environment variable configuration options and exit
    #[arg(long)]
    show_config_options: bool,
}

static ENV_VAR_OPTIONS: &str = "
CONFIGURATION
`nps` can be configured with environment variables. You can set these in
the configuration file of your shell, e.g. .bashrc/.zshrc

NIX_PACKAGE_SEARCH_FLIP
  Flip the order of matches? By default most relevant matches appear first.
  Flipping the order makes them appear last and is thus easier to read with
  long output.
    possible values: true, false
    default: false

NIX_PACKAGE_SEARCH_FOLDER
  In which folder is the cache located?
    possible values: path
    default: ${HOME}/.nix-package-search

NIX_PACKAGE_SEARCH_CACHE_FILE
  Name of the cache file
    possible values: filename
    default: nps.cache

NIX_PACKAGE_SEARCH_SHOW_PACKAGE_VERSION
  Show the PACKAGE_VERSION column
    possible values: true, false
    default: true

NIX_PACKAGE_SEARCH_SHOW_PACKAGE_DESCRIPTION
  Show the PACKAGE_DESCRIPTION column
    possible values: true, false
    default: true

NIX_PACKAGE_SEARCH_EXACT_COLOR
  Color of EXACT matches, match PACKAGE_NAME
    possible values: black, blue, green, red, cyan, magenta, yellow, white
    default: magenta

NIX_PACKAGE_SEARCH_DIRECT_COLOR
  Color of DIRECT matches, match PACKAGE_NAME-bar
    possible values: black, blue, green, red, cyan, magenta, yellow, white
    default: blue

NIX_PACKAGE_SEARCH_INDIRECT_COLOR
  Color of INDIRECT matches, match foo-PACKAGE_NAME-bar
    possible values: black, blue, green, red, cyan, magenta, yellow, white
    default: green

NIX_PACKAGE_SEARCH_COLOR_MODE
  show search matches in color
    possible values:
      - never:   Never show color
      - always:  Always show color
      - auto:    Only show color if stdout is in terminal, suppress if e.g. piped
    default: auto

NIX_PACKAGE_SEARCH_PRINT_SEPARATOR
  Separate matches with a newline?
    possible values: true, false
    default: true
";

/// Column name options
#[derive(Clone, Debug, ValueEnum)]
enum Columns {
    /// Show all columns
    All,
    /// Show only PACKAGE_NAME
    None,
    /// Also show PACKAGE_VERSION
    Version,
    /// Also show PACKAGE_DESCRIPTION
    Description,
}


fn print_formatted_option_help_text(help_text: &str) {
    let mut header = false;

    for line in help_text.lines() {
        if line.is_empty() {
            header = true;
            println!();
            continue;
        }
        if header {
            println!("{}", line.if_supports_color(Stdout, |text| text.bold()));
            header = false;
        } else {
            println!("{}", line);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let log_level = match cli.debug {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    // Set a supports-color override based on the variable passed in.
    match cli.color {
        ColorChoice::Always => owo_colors::set_override(true),
        ColorChoice::Auto => {}
        ColorChoice::Never => owo_colors::set_override(false),
    }

    env_logger::Builder::new().filter_level(log_level).init();

    if cli.show_config_options {
        print_formatted_option_help_text(ENV_VAR_OPTIONS);
        return;
    };

    println!("color: {:?}", cli.color);
    println!("columns: {:?}", cli.columns);
    println!("debug: {:?}", cli.debug);
    println!("flip: {:?}", cli.flip);
    println!("refresh: {:?}", cli.refresh);
    println!("separate: {:?}", cli.separate);

    log::trace!("trace");
    log::debug!("debug");
    log::info!("information");
    log::warn!("warning");
    log::error!("error");
}
