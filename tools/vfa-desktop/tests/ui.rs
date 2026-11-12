//! Headless callback proof, deliberately not native rendering/AT evidence.

use gpui_kit::test::TestWindowExt;
use gpui_kit::{Bounds, TestAppContext, WindowBounds, WindowOptions, base::Root, px, size};
use vfa_desktop::open_bootstrap;

#[gpui_kit::test]
fn bootstrap_click_changes_application_state(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let (handle, view) = cx.update(|cx| {
        let bounds = Bounds::centered(None, size(px(640.), px(480.)), cx);
        open_bootstrap(
            cx,
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
        )
        .expect("production bootstrap opens on the test platform")
    });
    assert!(handle.downcast::<Root>().is_some(), "Kit must mount Root");
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let control = window.find("bootstrap-probe");
        assert!(control.visible());
        assert_eq!(control.label(), Some("Verify desktop interaction"));
        assert_eq!(view.read(cx).activations, 0);
        window.click("bootstrap-probe", cx);
        // Label-only assertions would accept a disconnected callback; this cannot.
        assert_eq!(view.read(cx).activations, 1);
        window.click("bootstrap-probe", cx);
        assert_eq!(view.read(cx).activations, 2);
    })
    .expect("test window remains live");
}
