use std::path::PathBuf;

use ras_agent::{FolderScreenshotSink, StepScreenshotRequest, StepScreenshotSink};
use ras_cdp::ScreenshotFormat;
use ras_types::{AgentId, StepId};
use uuid::Uuid;

const PNG_BYTES: &[u8] = b"\x89PNG\r\n\x1a\n";

fn scratch_root() -> PathBuf {
    std::env::temp_dir().join(format!("ras-screenshot-{}", Uuid::now_v7()))
}

fn request(agent: AgentId, step: u32, format: ScreenshotFormat) -> StepScreenshotRequest {
    StepScreenshotRequest {
        agent,
        step: StepId(step),
        format,
    }
}

#[tokio::test]
async fn test_save_writes_png_under_agent_directory_and_returns_its_location() {
    let root = scratch_root();
    let sink = FolderScreenshotSink::new(&root);
    let agent = AgentId::new();

    let saved = sink
        .save(request(agent, 3, ScreenshotFormat::Png), PNG_BYTES)
        .await
        .expect("save");

    let expected = root.join(agent.0.to_string()).join("step-0003.png");
    assert_eq!(saved.location, expected.to_string_lossy());
    assert_eq!(saved.size_bytes, PNG_BYTES.len() as u64);
    assert_eq!(
        tokio::fs::read(&expected).await.expect("read back"),
        PNG_BYTES
    );

    tokio::fs::remove_dir_all(&root).await.expect("cleanup");
}

#[tokio::test]
async fn test_save_creates_missing_directories_for_a_fresh_root() {
    let root = scratch_root().join("nested").join("deeper");
    let sink = FolderScreenshotSink::new(&root);

    let saved = sink
        .save(request(AgentId::new(), 0, ScreenshotFormat::Png), PNG_BYTES)
        .await
        .expect("save");

    assert!(PathBuf::from(&saved.location).exists());

    tokio::fs::remove_dir_all(&root).await.expect("cleanup");
}

#[tokio::test]
async fn test_path_for_uses_jpeg_extension_when_format_is_jpeg() {
    let sink = FolderScreenshotSink::new("/tmp/ras-shots");
    let agent = AgentId::new();

    let path = sink.path_for(request(agent, 12, ScreenshotFormat::Jpeg));

    assert_eq!(
        path,
        PathBuf::from("/tmp/ras-shots")
            .join(agent.0.to_string())
            .join("step-0012.jpeg")
    );
}

#[tokio::test]
async fn test_steps_are_zero_padded_so_names_sort_lexicographically() {
    let sink = FolderScreenshotSink::new("/tmp/ras-shots");
    let agent = AgentId::new();

    let early = sink.path_for(request(agent, 2, ScreenshotFormat::Png));
    let late = sink.path_for(request(agent, 10, ScreenshotFormat::Png));

    assert!(early < late);
}
