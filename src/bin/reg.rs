use grep;
use termcolor;

fn main() {
    let corpus = "abcde
ab abcde ab
abcde";

    let matcher = grep::regex::RegexMatcherBuilder::new()
        .case_insensitive(false)
        .build("abCde")
        .unwrap();

    let exact_color: grep::printer::UserColorSpec = "match:fg:red".parse().unwrap();
    let exact_style: grep::printer::UserColorSpec = "match:style:bold".parse().unwrap();
    let exact_color_specs = grep::printer::ColorSpecs::new(&[exact_color, exact_style]);

    let bufwtr = termcolor::BufferWriter::stdout(termcolor::ColorChoice::Always);
    let mut exact_buffer = bufwtr.buffer();

    let mut exact_printer = grep::printer::StandardBuilder::new()
        .color_specs(exact_color_specs)
        .build(&mut exact_buffer);

    grep::searcher::SearcherBuilder::new()
        .line_number(false)
        .build()
        .search_slice(&matcher, &corpus.as_bytes(), exact_printer.sink(&matcher))
        .unwrap();

    let text = String::from_utf8(exact_buffer.into_inner()).unwrap();
    println!("X");
    println!("{}", text);
    println!("X");
}
