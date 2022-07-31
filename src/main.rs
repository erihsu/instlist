use clap::Parser;
use instlist::InstListAnalyzer;
use std::path::PathBuf;

/// instlist CLI arguments
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct InstlistArgs {
    /// given filelist
    #[clap(short = 'f')]
    pub filelist: PathBuf,
    /// top module name
    pub top: String,
}

impl Default for InstlistArgs {
    fn default() -> Self {
        Self {
            filelist: PathBuf::new(),
            top: String::new(),
        }
    }
}

fn main() {
    let args = InstlistArgs::parse();

    let mut analyzer = InstListAnalyzer::new(args.top);
    analyzer.parse_from_filelist(args.filelist);
    assert_eq!(analyzer.analyze_filelist(), true);
    analyzer.generate_instlist();

    // display result
    analyzer.list_result();
}
