use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::Parser;
use env_logger;
use grep::{printer, regex, searcher};
use home;
use log;
use std::{io::IsTerminal, io::Write, process::ExitCode};
use termcolor::WriteColor;

const CACHE_FOLDER_LOCATION: &str = ".nix-package-search";
const CACHE_FOLDER_FILE_NAME: &str = "nps.experimental.cache";

/// Find SEARCH_TERM in available nix packages and sort results by relevance
///
/// List up to three columns, the latter two being optional:
/// PACKAGE_NAME  [PACKAGE_VERSION]  [PACKAGE_DESCRIPTION]
///
/// Matches are sorted by type. Show 'exact' matches first, then 'direct' matches, and finally 'indirect' matches.
///
///   exact     SEARCH_TERM (in PACKAGE_NAME column)
///   direct    SEARCH_TERMbar (in PACKAGE_NAME column)
///   indirect  fooSEARCH_TERMbar (in any column)
#[derive(clap::Parser, Debug)]
#[command(author, version, verbatim_doc_comment, styles=styles())]
struct Cli {
    // default_value_t: value if flag (or env var) not present
    // default_missing_value: value if flag is present, but has no value
    //                        needs .num_args(0..N) and .require_equals(true)
    // require_equals: force `--option=val` syntax
    // env: read env var if flag not present
    // takes_values: accept values from command line
    // hide: hides the option from `-h`, those parameters are set via env vars
    /// Highlight search matches in color
    #[arg(
        short,
        long = "color",
        require_equals = true,
        visible_alias = "colour",
        default_value_t = clap::ColorChoice::Auto,
        default_missing_value = "clap::ColorChoice::Auto",
        env = "NIX_PACKAGE_SEARCH_COLOR_MODE"
        )]
    color: clap::ColorChoice,

    /// Choose columns to show
    #[arg(
        short = 'C',
        long = "columns",
        require_equals = true,
        default_value_t = ColumnsChoice::All,
        default_missing_value = "ColumnsChoice::All",
        value_enum,
        env = "NIX_PACKAGE_SEARCH_COLUMNS"
    )]
    columns: ColumnsChoice,

    /// Turn debugging information on
    ///
    /// Use up to four times for increased verbosity
    #[arg(
        short,
        long,
        action = clap::ArgAction::Count
    )]
    debug: u8,

    /// Flip the order of matches and sorting
    #[arg(
        short,
        long,
        require_equals = true,
        default_value_t = false,
        default_missing_value = "true",
        num_args = 0..=1,
        action = clap::ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_FLIP"
    )]
    flip: bool,

    /// Ignore case
    #[arg(
        short,
        long,
        require_equals = true,
        default_value_t = false,
        default_missing_value = "true",
        num_args = 0..=1,
        action = clap::ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_IGNORE_CASE"
    )]
    ignore_case: bool,

    /// Refresh package cache and exit
    #[arg(short, long)]
    refresh: bool,

    /// Separate match types with a newline
    #[arg(
        short,
        long,
        require_equals = true,
        default_value_t = false,
        default_missing_value = "true",
        num_args = 0..=1,
        action = clap::ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_PRINT_SEPARATOR"
    )]
    separate: bool,

    /// Search for any SEARCH_TERM in package names, description or versions
    #[arg(
        required_unless_present_any = ["show_config_options", "refresh"]
    )]
    search_term: Option<String>,

    /// Show environment variable configuration options and exit
    #[arg(long)]
    show_config_options: bool,

    // hidden vars, to be set via env vars
    /// Cache lives here
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value = home::home_dir().unwrap().join(CACHE_FOLDER_LOCATION).display().to_string(),
        value_parser = clap::value_parser!(std::path::PathBuf),
        env = "NIX_PACKAGE_SEARCH_FOLDER"
    )]
    search_folder: std::path::PathBuf,

    /// Color of EXACT matches, match SEARCH_TERM
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value_t = Colors::Magenta,
        value_enum,
        action = clap::ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_EXACT_COLOR"
    )]
    exact_color: Colors,

    /// Color of DIRECT matches, match SEARCH_TERMbar
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value_t = Colors::Blue,
        value_enum,
        action = clap::ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_DIRECT_COLOR"
    )]
    direct_color: Colors,

    /// Color of DIRECT matches, match fooSEARCH_TERMbar (or match other columns)
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value_t = Colors::Green,
        value_enum,
        action = clap::ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_INDIRECT_COLOR"
    )]
    indirect_color: Colors,
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

NIX_PACKAGE_SEARCH_COLUMNS
  Choose columns to show: PACKAGE_NAME plus any of PACKAGE_VERSION or
  PACKAGE_DESCRIPTION
    possible values: all, none, version, description
    default: all

NIX_PACKAGE_SEARCH_EXACT_COLOR
  Color of EXACT matches, match SEARCH_TERM in PACKAGE_NAME
    possible values: black, blue, green, red, cyan, magenta, yellow, white
    default: magenta

NIX_PACKAGE_SEARCH_DIRECT_COLOR
  Color of DIRECT matches, match SEARCH_TERMbar in PACKAGE_NAME
    possible values: black, blue, green, red, cyan, magenta, yellow, white
    default: blue

NIX_PACKAGE_SEARCH_INDIRECT_COLOR
  Color of INDIRECT matches, match fooSEARCH_TERMbar in any column
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
#[derive(Clone, Debug, clap::ValueEnum)]
enum ColumnsChoice {
    /// Show all columns
    All,
    /// Show only PACKAGE_NAME
    None,
    /// Also show PACKAGE_VERSION
    Version,
    /// Also show PACKAGE_DESCRIPTION
    Description,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum Colors {
    Black,
    Blue,
    Green,
    Red,
    Cyan,
    Magenta,
    Yellow,
    White,
}

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Red.on_default() | Effects::BOLD)
        .usage(AnsiColor::Red.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

fn print_formatted_option_help_text(
    help_text: &str,
    color_choice: termcolor::ColorChoice,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut header = false;

    let mut stdout = termcolor::StandardStream::stdout(color_choice);

    let help_text_with_folder = str::replace(
        help_text,
        "${HOME}/.nix-package-search",
        &home::home_dir()
            .unwrap()
            .join(".nix-package-search")
            .display()
            .to_string(),
    );

    for line in help_text_with_folder.lines() {
        if line.is_empty() {
            header = true;
            println!();
            continue;
        }
        if header {
            stdout.set_color(termcolor::ColorSpec::new().set_bold(true))?;
            writeln!(stdout, "{}", line)?;
            stdout.set_color(termcolor::ColorSpec::new().set_bold(false))?;
            header = false;
        } else {
            println!("{}", line);
        }
    }
    Ok(())
}

fn get_matches(
    search_term: &str,
    content: &str,
    ignore_case: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let matcher = regex::RegexMatcherBuilder::new()
        .case_insensitive(ignore_case)
        .build(search_term)?;
    let mut printer = printer::Standard::new_no_color(vec![]);
    searcher::SearcherBuilder::new()
        .line_number(false)
        .build()
        .search_slice(&matcher, &content.as_bytes(), printer.sink(&matcher))?;

    // into_inner gives us back the underlying writer we provided to
    // new_no_color, which is wrapped in a termcolor::NoColor. Thus, a second
    // into_inner gives us back the actual buffer.
    let output = String::from_utf8(printer.into_inner().into_inner())?;

    Ok(output)
}

fn convert_case(string: &str, ignore_case: bool) -> String {
    match ignore_case {
        true => string.to_lowercase(),
        false => string.to_string(),
    }
}

#[derive(Debug)]
struct Row {
    name: String,
    version: String,
    description: String,
}

#[derive(Debug)]
struct Matches {
    exact: Vec<Row>,
    direct: Vec<Row>,
    indirect: Vec<Row>,
}

fn print_matches(
    color_specs: printer::ColorSpecs,
    color_choice: termcolor::ColorChoice,
    joined_matches: String,
    matcher: &regex::RegexMatcher,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut printer = printer::StandardBuilder::new()
        .color_specs(color_specs)
        .build(termcolor::StandardStream::stdout(color_choice));
    searcher::SearcherBuilder::new()
        .line_number(false)
        .build()
        .search_slice(matcher, &joined_matches.as_bytes(), printer.sink(&matcher))?;
    Ok(())
}

fn assemble_string(
    row: Row,
    columns: &ColumnsChoice,
    name_padding: usize,
    version_padding: usize,
) -> String {
    match columns {
        ColumnsChoice::All => format!(
            "{:name_padding$}  {:version_padding$}  {}",
            row.name, row.version, row.description
        ),
        ColumnsChoice::Version => format!("{:name_padding$}  {}", row.name, row.version),
        ColumnsChoice::Description => format!("{:name_padding$}  {}", row.name, row.description),
        ColumnsChoice::None => format!("{}", row.name),
    }
}

fn sort_matches<'a>(
    raw_matches: String,
    color_choice: termcolor::ColorChoice,
    cli: Cli,
) -> Result<(), Box<dyn std::error::Error>> {
    let search_term = &cli.search_term.unwrap();
    let columns = cli.columns;
    let flip = cli.flip;
    let ignore_case = cli.ignore_case;
    let separate = cli.separate;

    let mut matches = Matches {
        exact: vec![],
        direct: vec![],
        indirect: vec![],
    };
    let mut name_lengths: Vec<usize> = vec![];
    let mut version_lengths: Vec<usize> = vec![];

    for line in raw_matches.lines() {
        let mut split_line: Vec<&str> = line.splitn(3, ' ').collect();
        while split_line.len() < 3 {
            split_line.push(""); // fill empty fields
        }
        let (name, version, description) = (
            split_line[0].to_string(),
            split_line[1].to_string(),
            split_line[2].to_string(),
        );

        name_lengths.push(name.len());
        version_lengths.push(version.len());

        let converted_search_term = &convert_case(&search_term, ignore_case);
        let converted_name = &convert_case(&name, ignore_case);

        let row = Row {
            name,
            version,
            description,
        };

        if converted_name == converted_search_term {
            matches.exact.push(row);
        } else if converted_name.starts_with(converted_search_term) {
            matches.direct.push(row);
        } else {
            matches.indirect.push(row);
        }
    }

    let name_padding = *name_lengths.iter().max().unwrap_or(&0);
    let version_padding = *version_lengths.iter().max().unwrap_or(&0);

    let mut exact: Vec<String> = vec![];
    let mut direct: Vec<String> = vec![];
    let mut indirect: Vec<String> = vec![];

    for row in matches.exact {
        exact.push(assemble_string(
            row,
            &columns,
            name_padding,
            version_padding,
        ));
    }

    for row in matches.direct {
        direct.push(assemble_string(
            row,
            &columns,
            name_padding,
            version_padding,
        ));
    }

    for row in matches.indirect {
        indirect.push(assemble_string(
            row,
            &columns,
            name_padding,
            version_padding,
        ));
    }

    if flip {
        exact.reverse();
        direct.reverse();
        indirect.reverse();
    }

    let matcher = regex::RegexMatcherBuilder::new()
        .case_insensitive(ignore_case)
        // Search for "search_term" OR any first character "^.", so we don't drop lines during the
        // coloring. A bit hacky. Not that we would want that, but I have no clue why the first
        // char char is not colored as a regex match as well. Magic?
        // TODO make less hacky
        .build(&format!("({}|^.)", search_term))?;

    let exact_color: printer::UserColorSpec = format!("match:fg:{:?}", cli.exact_color).parse()?;
    let direct_color: printer::UserColorSpec =
        format!("match:fg:{:?}", cli.direct_color).parse()?;
    let indirect_color: printer::UserColorSpec =
        format!("match:fg:{:?}", cli.indirect_color).parse()?;
    let exact_style: printer::UserColorSpec = "match:style:bold".parse()?;
    let direct_style: printer::UserColorSpec = "match:style:bold".parse()?;
    let indirect_style: printer::UserColorSpec = "match:style:bold".parse()?;
    let exact_color_specs = printer::ColorSpecs::new(&[exact_color, exact_style]);
    let direct_color_specs = printer::ColorSpecs::new(&[direct_color, direct_style]);
    let indirect_color_specs = printer::ColorSpecs::new(&[indirect_color, indirect_style]);

    match flip {
        true => {
            print_matches(
                indirect_color_specs,
                color_choice,
                indirect.join("\n"),
                &matcher,
            )?;
            if separate {
                println!();
            }
            print_matches(
                direct_color_specs,
                color_choice,
                direct.join("\n"),
                &matcher,
            )?;
            if separate {
                println!();
            }
            print_matches(exact_color_specs, color_choice, exact.join("\n"), &matcher)?;
        }
        false => {
            print_matches(exact_color_specs, color_choice, exact.join("\n"), &matcher)?;
            if separate {
                println!();
            }
            print_matches(
                direct_color_specs,
                color_choice,
                direct.join("\n"),
                &matcher,
            )?;
            if separate {
                println!();
            }
            print_matches(
                indirect_color_specs,
                color_choice,
                indirect.join("\n"),
                &matcher,
            )?;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let log_level = match cli.debug {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::new().filter_level(log_level).init();

    if cli.debug > 4 {
        log::error!("Max log level is 4, e.g. -dddd");
        std::process::exit(1)
    }

    log::trace!("Log level set to: {}", log_level);

    //dbg!(cli);
    //return ExitCode::SUCCESS;

    // Set a supports-color override based on the variable passed in.
    let color_choice = match cli.color {
        clap::ColorChoice::Always => {
            log::trace!("clap::ColorChoice set to Always");
            termcolor::ColorChoice::Always
        }
        clap::ColorChoice::Auto => {
            log::trace!("clap::ColorChoice request Auto");
            if std::io::stdout().is_terminal() {
                log::trace!("Running in terminal, clap::ColorChoice set to Auto");
                termcolor::ColorChoice::Auto
            } else {
                log::warn!("Not running in terminal, ColorCoice forced to Never");
                termcolor::ColorChoice::Never
            }
        }
        clap::ColorChoice::Never => {
            log::trace!("clap::ColorChoice set to Never");
            termcolor::ColorChoice::Never
        }
    };

    if cli.show_config_options {
        log::trace!("Show config options and exit");
        match print_formatted_option_help_text(ENV_VAR_OPTIONS, color_choice) {
            Ok(_) => (),
            Err(err) => {
                log::error!("Can't show config options: {}", err);
                return ExitCode::FAILURE;
            }
        };
        return ExitCode::SUCCESS;
    };

    let file_path: std::path::PathBuf = cli.search_folder.join(CACHE_FOLDER_FILE_NAME);

    let content = match std::fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(err) => {
            log::error!("Can't open file {}: {err}", &file_path.display());
            return ExitCode::FAILURE;
        }
    };

    let raw_matches = match get_matches(
        &cli.search_term.as_ref().unwrap(),
        &content,
        cli.ignore_case,
    ) {
        Ok(matches) => matches,
        Err(err) => {
            log::error!("Can't get matches: {err}");
            return ExitCode::FAILURE;
        }
    };

    match sort_matches(raw_matches, color_choice, cli) {
        Ok(result) => result,
        Err(err) => {
            log::error!("Can't sort matches: {err}");
            return ExitCode::FAILURE;
        }
    };

    return ExitCode::SUCCESS;
}
