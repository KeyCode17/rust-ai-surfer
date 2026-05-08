use std::path::PathBuf;

use ras_recording::RecorderPort;
use ras_recording::domain::recording::RecordingFormat;
use ras_recording::infrastructure::in_memory_recorder::InMemoryRecorder;

#[tokio::test]
async fn start_then_frame_then_stop() {
    let r = InMemoryRecorder::default();
    let s = r
        .start(&PathBuf::from("/tmp/x.gif"), RecordingFormat::Gif)
        .await
        .expect("start");
    assert_eq!(s.frame_count, 0);
    r.frame(b"abcd").await.expect("frame 1");
    r.frame(b"efgh").await.expect("frame 2");
    assert_eq!(r.frame_count().await, 2);
    let s = r.stop().await.expect("stop");
    assert_eq!(s.frame_count, 2);
}

#[tokio::test]
async fn frame_before_start_errors() {
    let r = InMemoryRecorder::default();
    assert!(r.frame(b"x").await.is_err());
}
