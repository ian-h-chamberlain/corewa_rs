use std::fs;
use std::path::{Path, PathBuf};

use normalize_line_endings::normalized;
use pretty_assertions::assert_eq;
use test_generator::test_resources;

use corewars_parser::Result as ParseResult;

#[test_resources("testdata/input/simple/*.redcode")]
#[test_resources("testdata/input/wilkie/*.redcode")]
#[test_resources("testdata/input/wilmoo/*.redcode")]
fn read_dir(input_file: &str) {
    // Workaround for the fact that `test_resources` paths are based on workspace Cargo.toml
    // https://github.com/frehberg/test-generator/issues/6
    let current_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(current_dir).unwrap();

    let input = fs::read_to_string(input_file)
        .unwrap_or_else(|err| panic!("Unable to read file {:?}: {:?}", input_file, err));

    let expected_out_file = PathBuf::from(input_file.replace("input", "expected_output"));
    if !expected_out_file.exists() {
        // TODO after #39 this shouldn't be needed
        eprintln!("No output file, skipping test");
        return;
    }

    let expected_output = fs::read_to_string(&expected_out_file)
        .map(|s| normalized(s.trim().chars()).collect::<String>())
        .unwrap_or_else(|err| panic!("Unable to read file {:?}: {:?}", input_file, err));

    let parsed_core = match corewars_parser::parse(&input) {
        ParseResult::Ok(core, _) => core,
        ParseResult::Err(e, _) => panic!("Parse error:\n{}", e),
    };

    let actual_output = parsed_core.to_string();

    let actual_lines: Vec<&str> = actual_output.trim().lines().collect();
    let expected_lines: Vec<&str> = expected_output.lines().collect();

    assert_eq!(expected_lines, actual_lines);
}
