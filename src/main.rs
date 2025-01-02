use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::Parser;
use env_logger;
use grep::{printer, regex, searcher};
use home;
use log;
use serde::Deserialize;
use serde_json;
use std::{io::IsTerminal, io::Write, process::Command, process::ExitCode};
use tempfile;
use termcolor;

const DEFAULTS: Defaults = Defaults {
    cache_folder: ".nix-package-search", // /home/USER/...
    cache_file: "nps.dev.cache",
    experimental: false,
    experimental_cache_file: "nps.experimental.dev.cache",
    color_mode: clap::ColorChoice::Auto,
    columns: ColumnsChoice::All,
    flip: false,
    ignore_case: true,
    print_separator: true,

    exact_color: Colors::Magenta,
    direct_color: Colors::Blue,
    indirect_color: Colors::Green,
};

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
#[command(
    author,
    version,
    verbatim_doc_comment,
    styles = styles(),
    after_long_help = option_help_text(ENV_VAR_OPTIONS)
)]
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
        default_value_t = DEFAULTS.color_mode,
        default_missing_value = "clap::ColorChoice::Auto",
        env = "NIX_PACKAGE_SEARCH_COLOR_MODE"
        )]
    color: clap::ColorChoice,

    /// Choose columns to show
    #[arg(
        short = 'C',
        long = "columns",
        require_equals = true,
        default_value_t = DEFAULTS.columns,
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

    /// Use experimental flakes
    #[arg(
        short,
        long,
        require_equals = true,
        default_value_t = DEFAULTS.experimental,
        default_missing_value = "true",
        num_args = 0..=1,
        action = clap::ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_EXPERIMENTAL"
    )]
    experimental: bool,

    /// Flip the order of matches and sorting
    #[arg(
        short,
        long,
        require_equals = true,
        default_value_t = DEFAULTS.flip,
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
        default_value_t = DEFAULTS.ignore_case,
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
        default_value_t = DEFAULTS.print_separator,
        default_missing_value = "true",
        num_args = 0..=1,
        action = clap::ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_PRINT_SEPARATOR"
    )]
    separate: bool,

    /// Search for any SEARCH_TERM in package names, description or versions
    #[arg(
        required_unless_present_any = ["refresh"]
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
        default_value = home::home_dir().unwrap().join(DEFAULTS.cache_folder).display().to_string(),
        value_parser = clap::value_parser!(std::path::PathBuf),
        env = "NIX_PACKAGE_SEARCH_CACHE_FOLDER"
    )]
    cache_folder: std::path::PathBuf,

    /// Cache file name
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value = DEFAULTS.cache_file,
        value_parser = clap::value_parser!(std::path::PathBuf),
        env = "NIX_PACKAGE_SEARCH_CACHE_FILE"
    )]
    cache_file: std::path::PathBuf,

    /// Experimental cache file name
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value = DEFAULTS.experimental_cache_file,
        value_parser = clap::value_parser!(std::path::PathBuf),
        env = "NIX_PACKAGE_SEARCH_EXPERIMENTAL_CACHE_FILE"
    )]
    experimental_cache_file: std::path::PathBuf,

    /// Color of EXACT matches, match SEARCH_TERM
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value_t = DEFAULTS.exact_color,
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
        default_value_t = DEFAULTS.direct_color,
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
        default_value_t = DEFAULTS.indirect_color,
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

NIX_PACKAGE_SEARCH_EXPERIMENTAL
  Use the experimental 'nix search' command.
  It pulls information from the nix flake registries instead of nix channels.
  This is useful if no channels are in use, or channels are not updated
  regularly.
    [default: {DEFAULT_EXPERIMENTAL}]
    [possible values: true, false]

NIX_PACKAGE_SEARCH_FLIP
  Flip the order of matches? By default most relevant matches appear below,
  which is easier to read with long output. Flipping shows most relevant
  matches on top.
    [default: {DEFAULT_FLIP}]
    [possible values: true, false]

NIX_PACKAGE_SEARCH_CACHE_FOLDER
  In which folder is the cache located?
    [default: {DEFAULT_CACHE_FOLDER}]
    [possible values: path]

NIX_PACKAGE_SEARCH_CACHE_FILE
  Name of the cache file
    [default: {DEFAULT_CACHE_FILE}]
    [possible values: filename]

NIX_PACKAGE_SEARCH_EXPERIMENTAL_CACHE_FILE
  Name of the cache file
    [default: {DEFAULT_EXPERIMENTAL_CACHE_FILE}]
    [possible values: filename]

NIX_PACKAGE_SEARCH_COLUMNS
  Choose columns to show: PACKAGE_NAME plus any of PACKAGE_VERSION or
  PACKAGE_DESCRIPTION
    [default: {DEFAULT_COLUMNS}]
    [possible values: all, none, version, description]

NIX_PACKAGE_SEARCH_EXACT_COLOR
  Color of EXACT matches, match SEARCH_TERM in PACKAGE_NAME
    [default: {DEFAULT_EXACT_COLOR}]
    [possible values: black, blue, green, red, cyan, magenta, yellow, white]

NIX_PACKAGE_SEARCH_DIRECT_COLOR
  Color of DIRECT matches, match SEARCH_TERMbar in PACKAGE_NAME
    [default: {DEFAULT_DIRECT_COLOR}]
    [possible values: black, blue, green, red, cyan, magenta, yellow, white]

NIX_PACKAGE_SEARCH_INDIRECT_COLOR
  Color of INDIRECT matches, match fooSEARCH_TERMbar in any column
    [default: {DEFAULT_INDIRECT_COLOR}]
    [possible values: black, blue, green, red, cyan, magenta, yellow, white]

NIX_PACKAGE_SEARCH_COLOR_MODE
  Show search matches in color
  auto: Only show color if stdout is in terminal, suppress if e.g. piped
    [default: {DEFAULT_COLOR_MODE}]
    [possible values: always, never, auto]

NIX_PACKAGE_SEARCH_PRINT_SEPARATOR
  Separate matches with a newline?
    [default: {DEFAULT_PRINT_SEPARATOR}]
    [possible values: true, false]

NIX_PACKAGE_SEARCH_IGNORE_CASE
  Search ignore capitalization for the search?
    [default: {DEFAULT_IGNORE_CASE}]
    [possible values: true, false]
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

struct Defaults<'a> {
    cache_folder: &'a str,
    cache_file: &'a str,
    experimental: bool,
    experimental_cache_file: &'a str,
    color_mode: clap::ColorChoice,
    columns: ColumnsChoice,
    flip: bool,
    ignore_case: bool,
    print_separator: bool,

    exact_color: Colors,
    direct_color: Colors,
    indirect_color: Colors,
}

fn option_help_text(help_text: &str) -> String {
    help_text
        .replace("{DEFAULT_EXPERIMENTAL}", &DEFAULTS.experimental.to_string())
        .replace("{DEFAULT_CACHE_FOLDER}", DEFAULTS.cache_folder)
        .replace("{DEFAULT_CACHE_FILE}", DEFAULTS.cache_file)
        .replace(
            "{DEFAULT_EXPERIMENTAL_CACHE_FILE}",
            DEFAULTS.experimental_cache_file,
        )
        .replace(
            "{DEFAULT_COLOR_MODE}",
            &DEFAULTS.color_mode.to_string().to_lowercase(),
        )
        .replace(
            "{DEFAULT_COLUMNS}",
            &format!("{:?}", DEFAULTS.columns).to_lowercase(),
        )
        .replace("{DEFAULT_FLIP}", &DEFAULTS.flip.to_string())
        .replace("{DEFAULT_IGNORE_CASE}", &DEFAULTS.ignore_case.to_string())
        .replace(
            "{DEFAULT_PRINT_SEPARATOR}",
            &DEFAULTS.print_separator.to_string(),
        )
        .replace(
            "{DEFAULT_EXACT_COLOR}",
            &format!("{:?}", DEFAULTS.exact_color).to_lowercase(),
        )
        .replace(
            "{DEFAULT_DIRECT_COLOR}",
            &format!("{:?}", DEFAULTS.direct_color).to_lowercase(),
        )
        .replace(
            "{DEFAULT_INDIRECT_COLOR}",
            &format!("{:?}", DEFAULTS.indirect_color).to_lowercase(),
        )
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
    search_term: String,
    columns: ColumnsChoice,
    flip: bool,
    ignore_case: bool,
    separate: bool,
    exact_color: Colors,
    direct_color: Colors,
    indirect_color: Colors,
) -> Result<(), Box<dyn std::error::Error>> {
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

    if flip == false {
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

    let exact_color: printer::UserColorSpec = format!("match:fg:{:?}", exact_color).parse()?;
    let direct_color: printer::UserColorSpec =
        format!("match:fg:{:?}", direct_color).parse()?;
    let indirect_color: printer::UserColorSpec =
        format!("match:fg:{:?}", indirect_color).parse()?;
    let exact_style: printer::UserColorSpec = "match:style:bold".parse()?;
    let direct_style: printer::UserColorSpec = "match:style:bold".parse()?;
    let indirect_style: printer::UserColorSpec = "match:style:bold".parse()?;
    let exact_color_specs = printer::ColorSpecs::new(&[exact_color, exact_style]);
    let direct_color_specs = printer::ColorSpecs::new(&[direct_color, direct_style]);
    let indirect_color_specs = printer::ColorSpecs::new(&[indirect_color, indirect_style]);

    match flip {
        true => {
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
        false => {
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
    }
    Ok(())
}

fn parse_json_to_cache(raw_output: &str) -> String {
    let parsed: std::collections::HashMap<String, Package> =
        serde_json::from_str(raw_output).unwrap();
    let mut result = vec![];
    for (_, package) in parsed.into_iter() {
        result.push(format!(
            "{} {} {}",
            package.pname, package.version, package.description
        ));
    }
    result.sort();
    result.join("\n")
}

#[derive(Debug, Deserialize)]
struct Package {
    pname: String,
    version: String,
    description: String,
}

fn refresh(experimental: bool, cache_folder: std::path::PathBuf, cache_file: std::path::PathBuf, experimental_cache_file: std::path::PathBuf) -> Result<(usize, String), Box<dyn std::error::Error>> {
    let output = match experimental {
        true => Command::new("nix")
            .arg("search")
            .arg("nixpkgs")
            .arg("^")
            .arg("--json")
            .output()?,
        false => Command::new("nix-env")
            .arg("-qaP")
            .arg("--description")
            .output()?,
    };

    let (stdout, _stderr) = (
        std::str::from_utf8(&output.stdout).unwrap(),
        std::str::from_utf8(&output.stderr).unwrap(),
    );

    // experimental=true fails, why? TODO
    //if stderr != "" {
    //    return Err(stderr.into());
    //}

    let cache_content = match experimental {
        true => parse_json_to_cache(stdout),
        false => stdout.to_string(),
    };

    // Create cache folder, if not exists
    std::fs::create_dir_all(
        home::home_dir()
            .unwrap()
            .join(std::path::PathBuf::from(&cache_folder)),
    )?;

    // Paths for cache folder and cache file
    let cache_folder_path = home::home_dir()
        .unwrap()
        .join(std::path::PathBuf::from(cache_folder));
    let cache_file_path = match experimental {
        true => &cache_folder_path.join(std::path::PathBuf::from(experimental_cache_file)),
        false => &cache_folder_path.join(std::path::PathBuf::from(cache_file)),
    };

    // Write first to a tmp file, then persist (move) it to destination
    let tempfile = tempfile::NamedTempFile::new_in(cache_folder_path)?;
    write!(&tempfile, "{}", cache_content)?;

    tempfile.persist(cache_file_path)?;

    let number_of_packages = cache_content.lines().count();
    let cache_file_path_string = cache_file_path.display().to_string();

    return Ok((number_of_packages, cache_file_path_string));
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

    if cli.refresh {
        match refresh(cli.experimental, cli.cache_folder, cli.cache_file, cli.experimental_cache_file) {
            Ok((number_of_packages, cache_file_path_string)) => {
                log::info!("Done. Cached info of {number_of_packages} packages in {cache_file_path_string}");
                return ExitCode::SUCCESS;
            }
            Err(err) => {
                log::error!("Can't refresh: {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    let file_path: std::path::PathBuf = cli.cache_folder.join(cli.experimental_cache_file);

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

    match sort_matches(raw_matches, color_choice, cli.search_term.unwrap(), cli.columns, cli.flip, cli.ignore_case, cli.separate, cli.exact_color, cli.direct_color, cli.indirect_color) {
        Ok(result) => result,
        Err(err) => {
            log::error!("Can't sort matches: {err}");
            return ExitCode::FAILURE;
        }
    };

    return ExitCode::SUCCESS;
}
