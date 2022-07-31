use instlist::InstListAnalyzer;

#[test]
fn test_analyze_demo() {
    let mut analyzer = InstListAnalyzer::new("complex_logic");
    analyzer.parse_from_filelist("testcase/complex_logic.f");
    assert_eq!(analyzer.analyze_filelist(), true);
}
