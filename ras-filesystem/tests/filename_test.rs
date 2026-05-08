use ras_filesystem::application::filename_validator::{parse_filename, sanitize};
use ras_filesystem::domain::file::FileExtension;

#[test]
fn parses_valid_names() {
    let (stem, ext) = parse_filename("notes.md").expect("valid");
    assert_eq!(stem, "notes");
    assert_eq!(ext, FileExtension::Md);
}

#[test]
fn parses_underscore_and_dash() {
    let (stem, ext) = parse_filename("my_data-file.csv").expect("valid");
    assert_eq!(stem, "my_data-file");
    assert_eq!(ext, FileExtension::Csv);
}

#[test]
fn rejects_missing_extension() {
    assert!(parse_filename("notes").is_err());
}

#[test]
fn rejects_unsupported_extension() {
    assert!(parse_filename("img.png").is_err());
}

#[test]
fn rejects_path_separators() {
    assert!(parse_filename("a/b.md").is_err());
    assert!(parse_filename("..\\bad.md").is_err());
}

#[test]
fn sanitize_replaces_spaces() {
    assert_eq!(sanitize("my file.csv"), "my_file.csv");
}

#[test]
fn sanitize_drops_disallowed() {
    assert_eq!(sanitize("a@b!c.md"), "abc.md");
}
