//! Native bootstrap only; catalog workflows and runtime ownership follow later.

use gpui_kit::component::button::Button;
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Context, Entity, IntoElement, ParentElement as _,
    Render, Styled as _, Window, WindowOptions, div,
};

/// A small observable control shared by real native and in-process bootstrap tests.
#[derive(Default)]
pub struct BootstrapView {
    pub activations: u32,
}

impl Render for BootstrapView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .p_4()
            .gap_4()
            .child("Vanguard Frontier Agentic — desktop bootstrap")
            .child(
                Button::new("bootstrap-probe")
                    .label("Verify desktop interaction")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.activations += 1;
                        cx.notify();
                    })),
            )
            .child(format!("Verified interactions: {}", self.activations))
    }
}

/// Kit mounts its Root around this content. Returning another Root would nest it.
pub fn open_bootstrap(
    cx: &mut App,
    options: WindowOptions,
) -> Result<(AnyWindowHandle, Entity<BootstrapView>), String> {
    gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| BootstrapView::default()))
        .map_err(|error| format!("Cannot open the native desktop window: {error:#}"))
}

pub fn run() -> Result<(), String> {
    use std::{cell::RefCell, rc::Rc};
    let error = Rc::new(RefCell::new(None));
    let observed = error.clone();
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            if let Err(failure) = open_bootstrap(cx, WindowOptions::default()) {
                *observed.borrow_mut() = Some(failure);
                cx.quit();
            }
        });
    match error.take() {
        Some(failure) => Err(failure),
        None => Ok(()),
    }
}
