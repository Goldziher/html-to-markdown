use clap::Parser;

#[derive(Parser)]
#[command(name = "html-to-markdown")]
#[command(about = "Convert HTML to Markdown", long_about = None)]
struct Cli {
    /// Input HTML file
    input: Option<String>,
}

fn main() {
    let _cli = Cli::parse();
    // TODO: Implementation
    println!("html-to-markdown CLI");
}
