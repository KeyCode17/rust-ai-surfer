use ras_filesystem::FileSystemPort;
use ras_filesystem::domain::file::{FileExtension, FileSystemFile};
use ras_filesystem::infrastructure::local_filesystem::LocalFileSystem;
use tempfile::tempdir;

#[tokio::test]
async fn write_then_read_round_trip() {
    let dir = tempdir().expect("tempdir");
    let fs = LocalFileSystem::new(dir.path().to_path_buf()).expect("new");
    fs.write(FileSystemFile {
        name: "notes.md".into(),
        extension: FileExtension::Md,
        bytes: b"# Hello".to_vec(),
    })
    .await
    .expect("write");
    let r = fs.read("notes.md").await.expect("read");
    assert_eq!(r.bytes, b"# Hello");
    assert_eq!(r.extension, FileExtension::Md);
}

#[tokio::test]
async fn write_csv_strips_blank_lines_on_disk() {
    let dir = tempdir().expect("tempdir");
    let fs = LocalFileSystem::new(dir.path().to_path_buf()).expect("new");
    fs.write(FileSystemFile {
        name: "data.csv".into(),
        extension: FileExtension::Csv,
        bytes: b"\n\na,b\n1,2\n\n".to_vec(),
    })
    .await
    .expect("write");
    let r = fs.read("data.csv").await.expect("read");
    let s = String::from_utf8(r.bytes).expect("utf8");
    assert_eq!(s, "a,b\n1,2");
}

#[tokio::test]
async fn list_returns_known_files_only() {
    let dir = tempdir().expect("tempdir");
    let fs = LocalFileSystem::new(dir.path().to_path_buf()).expect("new");
    fs.write(FileSystemFile {
        name: "a.md".into(),
        extension: FileExtension::Md,
        bytes: b"x".to_vec(),
    })
    .await
    .expect("write a");
    std::fs::write(dir.path().join("ignored.png"), b"x").expect("write png");
    let list = fs.list().await.expect("list");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "a.md");
}
