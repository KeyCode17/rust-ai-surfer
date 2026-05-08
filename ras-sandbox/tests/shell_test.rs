use std::time::Duration;

use ras_sandbox::ExecutionRequest;
use ras_sandbox::SandboxRunner;
use ras_sandbox::infrastructure::subprocess_runner::ShellSandbox;

#[tokio::test]
async fn echo_returns_stdout() {
    let r = ShellSandbox::default()
        .run(ExecutionRequest {
            script: "echo hello".into(),
            timeout: Duration::from_secs(5),
            env: Vec::new(),
        })
        .await
        .expect("run");
    assert_eq!(r.exit_code, 0);
    assert!(r.stdout.contains("hello"));
}

#[tokio::test]
async fn nonzero_exit_propagates() {
    let r = ShellSandbox::default()
        .run(ExecutionRequest {
            script: "exit 7".into(),
            timeout: Duration::from_secs(5),
            env: Vec::new(),
        })
        .await
        .expect("run");
    assert_eq!(r.exit_code, 7);
}
