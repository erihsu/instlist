use instlist::InstListAnalyzer;

fn main() {
    let mut analyzer = InstListAnalyzer::new("complex_logic");
    analyzer.parse_from_filelist("testcase/complex_logic.f");
    assert_eq!(analyzer.analyze_filelist(), true);
    analyzer.generate_instlist();

    for p in analyzer.instlist {
        println!("{:?}", p)
    }
}
