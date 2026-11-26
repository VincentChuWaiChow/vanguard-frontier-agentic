//! Owned watcher lifecycle on the host filesystem; no native UI claim.
use std::future::Future;
use std::time::Duration;
use tokio::time::timeout;
use vfa_tui::catalog::watcher::spawn_watcher;

#[tokio::test]
async fn shutdown_observes_worker_and_stops_notifications() {
    let root = tempfile::tempdir().unwrap();
    let (mut owner, mut events) = spawn_watcher(root.path(), None, &[]).unwrap();
    timeout(Duration::from_secs(5), owner.shutdown_async())
        .await
        .unwrap()
        .unwrap();
    std::fs::write(root.path().join("after.json"), "[]").unwrap();
    assert!(timeout(Duration::from_secs(5), events.recv())
        .await
        .unwrap()
        .is_none());
    // Completion can be observed repeatedly, with no second join or panic.
    owner.wait_stopped().await.unwrap();
}

#[tokio::test]
async fn receiver_closure_stops_worker_without_explicit_shutdown() {
    let root = tempfile::tempdir().unwrap();
    let (mut owner, events) = spawn_watcher(root.path(), None, &[]).unwrap();
    drop(events);
    timeout(Duration::from_secs(5), owner.wait_stopped())
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn cancelled_shutdown_retains_join_for_later_observation() {
    let root = tempfile::tempdir().unwrap();
    let (mut owner, _events) = spawn_watcher(root.path(), None, &[]).unwrap();
    // Poll once and cancel regardless of backend timing. Subsequent await must
    // still observe completion, not silently detach a taken JoinHandle.
    let mut shutdown = Box::pin(owner.shutdown_async());
    std::future::poll_fn(|cx| {
        assert!(shutdown.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    drop(shutdown);
    timeout(Duration::from_secs(5), owner.wait_stopped())
        .await
        .unwrap()
        .unwrap();
}
