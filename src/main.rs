use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{ArgAction, Parser, ValueEnum};
use env_logger::Builder;
use grep::{
    printer::{ColorSpecs, Standard, StandardBuilder, UserColorSpec},
    regex::RegexMatcherBuilder,
    searcher::SearcherBuilder,
};
use log::LevelFilter;
use serde::Deserialize;
use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{self, IsTerminal, Write},
    path::PathBuf,
    process::{Command, ExitCode},
    str,
};
use tempfile::NamedTempFile;
use termcolor::{Buffer, BufferWriter};

/// Default settings for `nps`.
///
/// They are also listed in the `-h`/`--help` commands.
const DEFAULTS: Defaults = Defaults {
    cache_folder: ".nix-package-search", // /home/USER/...
    cache_file: "nps.cache",             // not user settable
    experimental: false,
    experimental_cache_file: "nps.experimental.cache", // not user settable
    color_mode: clap::ColorChoice::Auto,
    columns: ColumnsChoice::All,
    flip: false,
    ignore_case: true,
    print_separator: true,
    quiet: false,

    exact_color: Colors::Magenta,
    direct_color: Colors::Blue,
    indirect_color: Colors::Green,
};

/// Find SEARCH_TERM in available nix packages and sort results by relevance
///
/// List up to three columns, the latter two being optional:
/// PACKAGE_NAME  <PACKAGE_VERSION>  <PACKAGE_DESCRIPTION>
///
/// Matches are sorted by type. Show 'exact' matches first, then 'direct' matches, and finally 'indirect' matches.
///
///   exact     SEARCH_TERM (in PACKAGE_NAME column)
///   direct    SEARCH_TERMbar (in PACKAGE_NAME column)
///   indirect  fooSEARCH_TERMbar (in any column)
#[derive(Parser, Debug)]
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
        num_args = 0..=1,
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
        num_args = 0..=1,
        env = "NIX_PACKAGE_SEARCH_COLUMNS"
    )]
    columns: ColumnsChoice,

    /// Turn debugging information on
    ///
    /// Use up to four times for increased verbosity
    #[arg(
        short,
        long,
        action = ArgAction::Count
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
        action = ArgAction::Set,
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
        action = ArgAction::Set,
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
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_IGNORE_CASE"
    )]
    ignore_case: bool,

    /// Suppress non-debug messages
    #[arg(
        short,
        long,
        require_equals = true,
        default_value_t = DEFAULTS.quiet,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_QUIET"
    )]
    quiet: bool,

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
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_PRINT_SEPARATOR"
    )]
    separate: bool,

    /// Search for any SEARCH_TERM in package names, description or versions
    #[arg(
        required_unless_present_any = ["refresh"]
    )]
    search_term: Option<String>,

    // hidden vars, to be set via env vars
    /// Cache lives here
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value = home::home_dir()
            .unwrap()  // We previously made sure this works.
            .join(DEFAULTS.cache_folder)
            .display()
            .to_string(),
        value_parser = clap::value_parser!(PathBuf),
        env = "NIX_PACKAGE_SEARCH_CACHE_FOLDER_ABSOLUTE_PATH"
    )]
    cache_folder: PathBuf,

    /// Color of EXACT matches, match SEARCH_TERM
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value_t = DEFAULTS.exact_color,
        value_enum,
        action = ArgAction::Set,
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
        action = ArgAction::Set,
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
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_INDIRECT_COLOR"
    )]
    indirect_color: Colors,
}

/// Help text for using environment variables for configuration.
///
/// Contains template items that still need to be replaced.
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

NIX_PACKAGE_SEARCH_CACHE_FOLDER_ABSOLUTE_PATH
  Absolute path of the cache folder
    [default: {DEFAULT_CACHE_FOLDER}]
    [possible values: path]

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

NIX_PACKAGE_SEARCH_QUIET
  Suppress non-debug messages?
    [default: {DEFAULT_QUIET}]
    [possible values: true, false]

NIX_PACKAGE_SEARCH_IGNORE_CASE
  Search ignore capitalization for the search?
    [default: {DEFAULT_IGNORE_CASE}]
    [possible values: true, false]
";

/// Column name options
#[derive(Clone, Debug, ValueEnum)]
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

/// Allowed values for coloring output.
#[derive(Debug, Clone, ValueEnum)]
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

/// Format to parse JSON package info into
#[derive(Debug, Deserialize)]
struct Package {
    // we are not using `pname`
    version: String,
    description: String,
}

/// Defines possible default settings.
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
    quiet: bool,

    exact_color: Colors,
    direct_color: Colors,
    indirect_color: Colors,
}

/// Supply Styles for colored help output.
fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Red.on_default() | Effects::BOLD)
        .usage(AnsiColor::Red.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

/// Replace template items in long help text with default settings.
fn option_help_text(help_text: &str) -> String {
    help_text
        .replace("{DEFAULT_EXPERIMENTAL}", &DEFAULTS.experimental.to_string())
        .replace(
            "{DEFAULT_CACHE_FOLDER}",
            &home::home_dir()
                .unwrap() // We previously made sure this works.
                .join(DEFAULTS.cache_folder)
                .display()
                .to_string(),
        )
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
            "{DEFAULT_QUIET}",
            &DEFAULTS.quiet.to_string(),
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

/// Find matches from cache file
fn get_matches(cli: &Cli, content: &str) -> Result<String, Box<dyn Error>> {
    let search_term = cli
        .search_term
        .as_ref()
        .ok_or("Can't get search term as ref")?;

    // Matcher to find search term in rows
    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(cli.ignore_case)
        .build(search_term)
        .map_err(|err| format!("Can't build regex: {err}"))?;
    // Printer collects matching rows in a Vec
    let mut printer = Standard::new_no_color(vec![]);

    // Execute search and collect output
    SearcherBuilder::new()
        .line_number(false)
        .build()
        .search_slice(&matcher, content.as_bytes(), printer.sink(&matcher))
        .map_err(|err| format!("Can't build searcher: {err}"))?;

    // into_inner gives us back the underlying writer we provided to
    // new_no_color, which is wrapped in a termcolor::NoColor. Thus, a second
    // into_inner gives us back the actual buffer.
    let output = String::from_utf8(printer.into_inner().into_inner())
        .map_err(|err| format!("Can't parse printer string: {err}"))?;

    Ok(output)
}

/// Case converter for case-insensitive searches
fn convert_case(string: &str, ignore_case: bool) -> String {
    match ignore_case {
        true => string.to_lowercase(),
        false => string.to_string(),
    }
}

type MatchVecs = (Vec<String>, Vec<String>, Vec<String>);

/// Sort matches into match types and pad the lines to aligned columns
fn sort_and_pad_matches(cli: &Cli, raw_matches: String) -> Result<MatchVecs, Box<dyn Error>> {
    let search_term = cli
        .search_term
        .as_ref()
        .ok_or("Can't get search term as ref")?;

    let mut name_lengths: Vec<usize> = vec![];
    let mut version_lengths: Vec<usize> = vec![];

    for line in raw_matches.lines() {
        let split_line: Vec<&str> = line.splitn(3, ' ').collect();

        // Try to get a split_line element: `.get()`,
        // use &"" if missing: `.unwrap_or(&"")`,
        // and append lengths `.len()` to *_lengths vectors.
        #[allow(clippy::get_first)]
        name_lengths.push(split_line.get(0).unwrap_or(&"").len());
        version_lengths.push(split_line.get(1).unwrap_or(&"").len());
    }

    // Mininum cell size will be the largest contained string
    let name_padding = *name_lengths.iter().max().unwrap_or(&0);
    let version_padding = *version_lengths.iter().max().unwrap_or(&0);

    let mut padded_matches_exact: Vec<String> = vec![];
    let mut padded_matches_direct: Vec<String> = vec![];
    let mut padded_matches_indirect: Vec<String> = vec![];

    for line in raw_matches.lines() {
        let split_line: Vec<&str> = line.splitn(3, ' ').collect();

        #[allow(clippy::get_first)] // supress clippy warning for this block
        let name = split_line.get(0).unwrap_or(&"");
        let version = split_line.get(1).unwrap_or(&"");
        let description = split_line.get(2).unwrap_or(&"");

        let assembled_line = match &cli.columns {
            ColumnsChoice::All => format!(
                "{:name_padding$}  {:version_padding$}  {}",
                name, version, description
            ),
            ColumnsChoice::Version => format!("{:name_padding$}  {}", name, version),
            ColumnsChoice::Description => format!("{:name_padding$}  {}", name, description),
            ColumnsChoice::None => format!("{} ", name),
        };

        // Handle case-insensitive, if requested
        let converted_search_term = &convert_case(search_term, cli.ignore_case);
        let converted_name = &convert_case(name, cli.ignore_case);

        // Package names from channels are prepended with "nixos." or "nixpgks."
        match cli.experimental {
            true => {
                if converted_name == converted_search_term {
                    padded_matches_exact.push(assembled_line);
                } else if converted_name.starts_with(converted_search_term) {
                    padded_matches_direct.push(assembled_line);
                } else {
                    padded_matches_indirect.push(assembled_line);
                }
            }
            false => {
                if converted_name == &("nixos.".to_owned() + converted_search_term)
                    || converted_name == &("nixpkgs.".to_owned() + converted_search_term)
                {
                    padded_matches_exact.push(assembled_line);
                } else if converted_name.starts_with(&("nixos.".to_owned() + converted_search_term))
                    || converted_name.starts_with(&("nixpkgs.".to_owned() + converted_search_term))
                {
                    padded_matches_direct.push(assembled_line);
                } else {
                    padded_matches_indirect.push(assembled_line);
                }
            }
        }
    }

    Ok((
        padded_matches_exact,
        padded_matches_direct,
        padded_matches_indirect,
    ))
}

/// Color the search term in different match types
fn color_matches(
    cli: &Cli,
    sorted_padded_matches: MatchVecs,
    color_choice: termcolor::ColorChoice,
) -> Result<[Buffer; 3], Box<dyn Error>> {
    let (mut padded_matches_exact, mut padded_matches_direct, mut padded_matches_indirect) =
        sorted_padded_matches;
    let search_term = cli
        .search_term
        .as_ref()
        .ok_or("Can't get search term as ref")?;

    // Defining different colors for different match types
    let exact_color: UserColorSpec = format!("match:fg:{:?}", &cli.exact_color).parse()?;
    let direct_color: UserColorSpec = format!("match:fg:{:?}", &cli.direct_color).parse()?;
    let indirect_color: UserColorSpec = format!("match:fg:{:?}", &cli.indirect_color).parse()?;

    // Font styles for match types
    let exact_style: UserColorSpec = "match:style:bold".parse()?;
    let direct_style: UserColorSpec = "match:style:bold".parse()?;
    let indirect_style: UserColorSpec = "match:style:bold".parse()?;

    // Combining colors and styles to ColorSpecs
    let exact_color_specs = ColorSpecs::new(&[exact_color, exact_style]);
    let direct_color_specs = ColorSpecs::new(&[direct_color, direct_style]);
    let indirect_color_specs = ColorSpecs::new(&[indirect_color, indirect_style]);

    // Create buffers to write colored output into
    let bufwtr = BufferWriter::stdout(color_choice);
    let mut exact_buffer = bufwtr.buffer();
    let mut direct_buffer = bufwtr.buffer();
    let mut indirect_buffer = bufwtr.buffer();

    // Printers print to the above buffers
    let mut exact_printer = StandardBuilder::new()
        .color_specs(exact_color_specs)
        .build(&mut exact_buffer);
    let mut direct_printer = StandardBuilder::new()
        .color_specs(direct_color_specs)
        .build(&mut direct_buffer);
    let mut indirect_printer = StandardBuilder::new()
        .color_specs(indirect_color_specs)
        .build(&mut indirect_buffer);

    // Matcher to color `search_term`
    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(cli.ignore_case)
        .build(search_term)
        .map_err(|err| format!("Can't build regex: {err}"))?;

    // Matcher to find _everything_, so lines without matches are still printed.
    // This can happen if certain columns are missing.
    let matcher_all = RegexMatcherBuilder::new().build(".*")?;

    // Let's have the top results at the bottom by default
    if !cli.flip {
        padded_matches_exact.reverse();
        padded_matches_direct.reverse();
        padded_matches_indirect.reverse();
    }

    // Coloring and printing to buffers
    SearcherBuilder::new()
        .line_number(false)
        .build()
        .search_slice(
            &matcher_all,
            padded_matches_exact.join("\n").as_bytes(),
            exact_printer.sink(&matcher),
        )
        .map_err(|err| format!("Can't build searcher: {err}"))?;
    SearcherBuilder::new()
        .line_number(false)
        .build()
        .search_slice(
            &matcher_all,
            padded_matches_direct.join("\n").as_bytes(),
            direct_printer.sink(&matcher),
        )
        .map_err(|err| format!("Can't build searcher: {err}"))?;
    SearcherBuilder::new()
        .line_number(false)
        .build()
        .search_slice(
            &matcher_all,
            padded_matches_indirect.join("\n").as_bytes(),
            indirect_printer.sink(&matcher),
        )
        .map_err(|err| format!("Can't build searcher: {err}"))?;

    Ok([exact_buffer, direct_buffer, indirect_buffer])
}

/// Print matches to screen in correct ordering
fn print_matches(cli: &Cli, colored_matches: [Buffer; 3]) -> Result<(), Box<dyn Error>> {
    // Assemble match type string segments
    let mut out: Vec<String> = vec![];
    for buffer in colored_matches.into_iter() {
        let content = String::from_utf8(buffer.into_inner())
            .map_err(|err| format!("Can't get string from buffer: {err}"))?;
        if !content.is_empty() {
            out.push(content);
        }
    }

    if !cli.flip {
        out.reverse();
    }

    // Use newlines as separators, if requested
    let separator = match cli.separate {
        true => "\n".to_string(),
        false => "".to_string(),
    };
    // BufferWriter introduces a newline that we need to trim for some reason
    writeln!(io::stdout(), "{}", &out.join(&separator).trim())
        .map_err(|err| format!("Can't write to stdout: {err}"))?;

    Ok(())
}

/// Parse package info from JSON to (NAME VERSION DESCRIPTION) lines
fn parse_json_to_lines(raw_output: &str) -> Result<String, Box<dyn Error>> {
    // Load JSON package info into a HashMap
    let parsed: HashMap<String, Package> =
        serde_json::from_str(raw_output).map_err(|err| format!("Can't parse JSON: {err}"))?;

    let mut lines = vec![];
    for (name_string, package) in parsed.into_iter() {
        // `name_string` is, for example, "legacyPackages.x86_64-linux.auctex"
        // Keep everything after the second '.' to get the package "name".
        // This is different from package.pname, which contains the name
        // of the executable, which can be different from the package name.
        let name_vec: Vec<&str> = name_string.splitn(3, '.').collect();
        let name = name_vec.get(2).ok_or("Can't get package name from JSON.")?;
        lines.push(format!(
            "{} {} {}",
            name, package.version, package.description
        ));
    }
    lines.sort();
    Ok(lines.join("\n"))
}

/// Fetch new package info and write to cache file
fn refresh(experimental: bool, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    log::info!("Refreshing cache");

    let cache_folder = file_path
        .parent()
        .ok_or("Can't get cache folder from file path")?;
    log::trace!("file_path: {:?}", file_path);

    let output = match experimental {
        true => Command::new("nix")
            .arg("search")
            .arg("nixpkgs")
            .arg("^")
            .arg("--json")
            .output()
            .map_err(|err| format!("`nix search` failed: {err}"))?,
        false => Command::new("nix-env")
            .arg("-qaP")
            .arg("--description")
            .output()
            .map_err(|err| format!("`nix-env` failed: {err}"))?,
    };

    log::trace!("finished cli command");

    let (stdout, stderr) = (
        str::from_utf8(&output.stdout)
            .map_err(|err| format!("Can't convert stdout to UTF8: {err}"))?,
        str::from_utf8(&output.stderr)
            .map_err(|err| format!("Can't convert stderr to UTF8: {err}"))?,
    );

    log::trace!("stdout.len(): {}", stdout.len());
    log::trace!("stderr.len(): {}", stderr.len());

    // Report warnings if stderr looks bad
    let mut first_error = true;
    for line in stderr.lines() {
        // ignore standard logging to stderr
        if !line.starts_with("evaluating") {
            if first_error {
                log::warn!("These warnings were encountered during cache refresh (START)");
                first_error = false;
            }
            log::warn!("> {}", line);
        }
    }
    if !first_error {
        log::warn!("These warnings were encountered during cache refresh (END)");
    }

    // Throw error if cache is too small
    if stdout.len() < 10_000 {
        log::error!("Only {} lines in cache.", stdout.len());
        return Err("Cache seems too small. Run with `-d` flag for more information.".into());
    }

    let cache_content = match experimental {
        true => parse_json_to_lines(stdout).map_err(|err| format!("Can't parse JSON: {err}"))?,
        false => {
            // Replace in every line the first two series of whitespaces with single spaces
            let re = regex::RegexBuilder::new(r"^([^ ]+) +([^ ]+) +(.*)$")
                .multi_line(true)
                .build()
                .unwrap();
            re.replace_all(stdout, "$1 $2 $3").to_string()
        }
    };

    log::trace!("trying to create folder: {:?}", cache_folder);
    // Create cache folder, if not exists
    fs::create_dir_all(cache_folder).map_err(|err| format!("Can't create folder: {err}"))?;
    log::trace!("folder created");

    log::trace!("cache_folder: {:?}", cache_folder);
    log::trace!("file_path: {:?}", &file_path);

    // Atomic Writing: Write first to a tmp file, then persist (move) it to destination
    let tempfile = NamedTempFile::new_in(cache_folder)
        .map_err(|err| format!("Can't create temp file: {err}"))?;
    log::trace!("tempfile: {:?}", &tempfile);
    log::trace!("trying to write tempfile");
    write!(&tempfile, "{}", cache_content)
        .map_err(|err| format!("Can't write to temp file: {err}"))?;
    log::trace!("tempfile written");

    tempfile
        .persist(file_path)
        .map_err(|err| format!("Can't persist temp file: {err}"))?;
    log::trace!("tempfile persisted");

    let number_of_packages = cache_content.lines().count();
    let cache_file_path_string = format!("{:?}", file_path);

    log::info!("Done. Cached info of {number_of_packages} packages in {cache_file_path_string}");

    Ok(())
}

fn main() -> ExitCode {
    // Get home dir errors out of the way, since clap can't propagate errors
    // from `derive`.
    let home = home::home_dir();
    if home.is_none() || home == Some("".into()) {
        Builder::new().filter_level(LevelFilter::Trace).init();
        log::error!("Can't find home dir.");
        return ExitCode::FAILURE;
    }
    let cli = Cli::parse();

    let log_level = match cli.debug {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    Builder::new().filter_level(log_level).init();

    if cli.debug > 4 {
        log::error!("Max log level is 4, e.g. -dddd");
        return ExitCode::FAILURE;
    }

    log::debug!("Log level set to: {}", log_level);

    // Set a supports-color override based on the variable passed in.
    let color_choice = match cli.color {
        clap::ColorChoice::Always => {
            log::debug!("clap::ColorChoice set to Always");
            termcolor::ColorChoice::Always
        }
        clap::ColorChoice::Auto => {
            log::debug!("clap::ColorChoice request Auto");
            if io::stdout().is_terminal() {
                log::debug!("Running in terminal, clap::ColorChoice set to Auto");
                termcolor::ColorChoice::Auto
            } else {
                log::warn!("Not running in terminal, clap::ColorCoice forced to Never");
                termcolor::ColorChoice::Never
            }
        }
        clap::ColorChoice::Never => {
            log::debug!("clap::ColorChoice set to Never");
            termcolor::ColorChoice::Never
        }
    };

    let cache_file = PathBuf::from(DEFAULTS.cache_file);
    let experimental_cache_file = PathBuf::from(DEFAULTS.experimental_cache_file);

    log::trace!("cache_file: {:?}", cache_file);
    log::trace!("experimental_cache_file: {:?}", experimental_cache_file);

    let file_path: PathBuf = match cli.experimental {
        true => cli.cache_folder.join(&experimental_cache_file),
        false => cli.cache_folder.join(&cache_file),
    };

    log::trace!("file_path: {:?}", file_path);

    let cache_file_exists = file_path.exists();

    log::trace!("cache_file_exists: {}", cache_file_exists);
    log::trace!("cli.refresh: {}", cli.refresh);

    // Refresh cache with new info?
    if cli.refresh || !cache_file_exists {
        log::trace!("inside if");
        match refresh(cli.experimental, &file_path) {
            Ok(_) => {
                if cli.refresh {
                    return ExitCode::SUCCESS;
                }
            }
            Err(err) => {
                log::error!("Can't refresh cache: {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(err) => {
            log::error!("Can't open file {}: {err}", &file_path.display());
            return ExitCode::FAILURE;
        }
    };

    let raw_matches = match get_matches(&cli, &content) {
        Ok(raw_matches) => raw_matches,
        Err(err) => {
            log::error!("Can't get matches: {err}");
            return ExitCode::FAILURE;
        }
    };
    if raw_matches.is_empty() {
        return ExitCode::FAILURE;
    }

    let sorted_padded_matches = match sort_and_pad_matches(&cli, raw_matches) {
        Ok(sorted_padded_matches) => sorted_padded_matches,
        Err(err) => {
            log::error!("Can't sort matches: {err}");
            return ExitCode::FAILURE;
        }
    };

    let colored_matches = match color_matches(&cli, sorted_padded_matches, color_choice) {
        Ok(colored_matches) => colored_matches,
        Err(err) => {
            log::error!("Can't color matches: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = print_matches(&cli, colored_matches) {
        log::error!("Can't print matches: {err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_get_matches() {
        init();

        let cli = Cli::try_parse_from(vec!["nps", "second"]).unwrap();
        let content = "\
            the first line\n\
            the second line\n\
            the third line\
            ";
        let matches = get_matches(&cli, content).unwrap();

        assert_eq!(matches, "the second line\n");
    }

    #[test]
    fn test_convert_case() {
        init();

        let test_string = "abCDef";

        assert_eq!(convert_case(test_string, false), "abCDef");
        assert_eq!(convert_case(test_string, true), "abcdef");
    }

    #[test]
    fn test_sort_and_pad_matches() {
        init();

        let cli_all_columns = Cli::try_parse_from(vec!["nps", "-e=true", "mypackage"]).unwrap();
        let cli_no_other_columns =
            Cli::try_parse_from(vec!["nps", "-e=true", "-C=none", "mypackage"]).unwrap();
        let cli_version_column =
            Cli::try_parse_from(vec!["nps", "-e=true", "-C=version", "mypackage"]).unwrap();
        let cli_description_column =
            Cli::try_parse_from(vec!["nps", "-e=true", "-C=description", "mypackage"]).unwrap();
        let matches = "\
            mypackage v1 my package description\n\
            myotherpackage v2 has description as well\n\
            mypackage_extension v3 words words\n\
            mypackage_extension_2 v4 words words w0rds\n\
            mylastpackage v5.0.0 is not mypackage\
            "
        .to_string();

        let exact_matches_all_columns = "\
            mypackage              v1      my package description\
            ";
        let direct_matches_all_columns = "\
            mypackage_extension    v3      words words\n\
            mypackage_extension_2  v4      words words w0rds\
            ";
        let indirect_matches_all_columns = "\
            myotherpackage         v2      has description as well\n\
            mylastpackage          v5.0.0  is not mypackage\
            ";

        let exact_matches_no_other_columns = "\
            mypackage \
            ";
        let direct_matches_no_other_columns = "\
            mypackage_extension \n\
            mypackage_extension_2 \
            ";
        let indirect_matches_no_other_columns = "\
            myotherpackage \n\
            mylastpackage \
            ";

        let exact_matches_version_column = "\
            mypackage              v1\
            ";
        let direct_matches_version_column = "\
            mypackage_extension    v3\n\
            mypackage_extension_2  v4\
            ";
        let indirect_matches_version_column = "\
            myotherpackage         v2\n\
            mylastpackage          v5.0.0\
            ";

        let exact_matches_description_column = "\
            mypackage              my package description\
            ";
        let direct_matches_description_column = "\
            mypackage_extension    words words\n\
            mypackage_extension_2  words words w0rds\
            ";
        let indirect_matches_description_column = "\
            myotherpackage         has description as well\n\
            mylastpackage          is not mypackage\
            ";

        let sorted_and_padded_all_columns =
            sort_and_pad_matches(&cli_all_columns, matches.clone()).unwrap();
        let sorted_and_padded_no_other_columns =
            sort_and_pad_matches(&cli_no_other_columns, matches.clone()).unwrap();
        let sorted_and_padded_version_column =
            sort_and_pad_matches(&cli_version_column, matches.clone()).unwrap();
        let sorted_and_padded_description_column =
            sort_and_pad_matches(&cli_description_column, matches).unwrap();

        assert_eq!(
            exact_matches_all_columns,
            sorted_and_padded_all_columns.0.join("\n")
        );
        assert_eq!(
            direct_matches_all_columns,
            sorted_and_padded_all_columns.1.join("\n")
        );
        assert_eq!(
            indirect_matches_all_columns,
            sorted_and_padded_all_columns.2.join("\n")
        );

        assert_eq!(
            exact_matches_no_other_columns,
            sorted_and_padded_no_other_columns.0.join("\n")
        );
        assert_eq!(
            direct_matches_no_other_columns,
            sorted_and_padded_no_other_columns.1.join("\n")
        );
        assert_eq!(
            indirect_matches_no_other_columns,
            sorted_and_padded_no_other_columns.2.join("\n")
        );

        assert_eq!(
            exact_matches_version_column,
            sorted_and_padded_version_column.0.join("\n")
        );
        assert_eq!(
            direct_matches_version_column,
            sorted_and_padded_version_column.1.join("\n")
        );
        assert_eq!(
            indirect_matches_version_column,
            sorted_and_padded_version_column.2.join("\n")
        );

        assert_eq!(
            exact_matches_description_column,
            sorted_and_padded_description_column.0.join("\n")
        );
        assert_eq!(
            direct_matches_description_column,
            sorted_and_padded_description_column.1.join("\n")
        );
        assert_eq!(
            indirect_matches_description_column,
            sorted_and_padded_description_column.2.join("\n")
        );
    }

    #[test]
    fn test_parse_json_to_lines() -> Result<(), Box<dyn Error>> {
        init();

        let json = "{\
            \"legacyPackages.x86_64-linux.mypackage\": {\
            \"description\":\"i describe\",\
            \"pname\":\"mypackagebinary\",\
            \"version\":\"old\"},\
            \
            \"legacyPackages.x86_64-linux.myotherpackage\": {\
            \"description\":\"i also describe\",\
            \"pname\":\"myotherpackagebinary\",\
            \"version\":\"fresh\"}\
            }";
        let desired_output = "\
            myotherpackage fresh i also describe\n\
            mypackage old i describe\
            ";
        let parsed = parse_json_to_lines(json)?;

        assert_eq!(parsed, desired_output);
        Ok(())
    }

    #[test]
    fn test_color_matches() {
        init();

        let cli = Cli::try_parse_from(vec!["nps", "-e=true", "mypackage"]).unwrap();
        let exact_matches = vec!["mypackage             v1     my package description".to_string()];
        let direct_matches = vec![
            "mypackage_extension   v1     my package description".to_string(),
            "mypackage_extension_2 v1.0.1 my package description".to_string(),
        ];
        let indirect_matches = vec![
            "mylastpackage         v5.0.0 is not mypackage".to_string(),
            "mylastpackage_2       v1     is not mypackage either".to_string(),
        ];

        let expect_color = [
            "\u{1b}[0m\u{1b}[1m\u{1b}[35mmypackage\u{1b}[0m             v1     my package description\n",
            "\u{1b}[0m\u{1b}[1m\u{1b}[34mmypackage\u{1b}[0m_extension_2 v1.0.1 my package description\n\
                \u{1b}[0m\u{1b}[1m\u{1b}[34mmypackage\u{1b}[0m_extension   v1     my package description\n",
            "mylastpackage_2       v1     is not \u{1b}[0m\u{1b}[1m\u{1b}[32mmypackage\u{1b}[0m either\n\
                mylastpackage         v5.0.0 is not \u{1b}[0m\u{1b}[1m\u{1b}[32mmypackage\u{1b}[0m\n",
        ];
        let expect_no_color = [
            "mypackage             v1     my package description\n",
            "mypackage_extension_2 v1.0.1 my package description\n\
                mypackage_extension   v1     my package description\n",
            "mylastpackage_2       v1     is not mypackage either\n\
                mylastpackage         v5.0.0 is not mypackage\n",
        ];

        let matches = (exact_matches, direct_matches, indirect_matches);

        let colored_matches_color =
            color_matches(&cli, matches.clone(), termcolor::ColorChoice::Always).unwrap();
        let colored_matches_no_color =
            color_matches(&cli, matches, termcolor::ColorChoice::Never).unwrap();

        for (expect, output) in std::iter::zip(
            [expect_color, expect_no_color].concat(),
            [colored_matches_color, colored_matches_no_color].concat(),
        ) {
            assert_eq!(expect, String::from_utf8(output.into_inner()).unwrap());
        }
    }
}
