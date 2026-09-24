//! Optional Windows administrator prompt and shared restart action.
use crate::elevation::RestartOutcome;
use gpui::*;
use gpui_component::{
    Disableable, WindowExt,
    button::{Button, ButtonVariants},
};

#[derive(Default)]
struct AdministratorState {
    required: bool,
    restarting: bool,
}
impl Global for AdministratorState {}

pub fn init(cx: &mut App) {
    let elevated = crate::elevation::is_elevated().unwrap_or_else(|error| {
        log::warn!("Could not read process elevation: {error}");
        false
    });
    cx.set_global(AdministratorState {
        required: !elevated,
        restarting: false,
    });
}

pub fn required(cx: &App) -> bool {
    cx.global::<AdministratorState>().required
}

pub fn restart_button(id: &'static str, cx: &App) -> Button {
    super::button::standard(id, cx)
        .label(crate::i18n::tr("Restart as administrator"))
        .disabled(cx.global::<AdministratorState>().restarting)
        .on_click(|_, window, cx| restart(window, cx))
}

pub fn show_startup_prompt(window: &mut Window, cx: &mut App) {
    if !required(cx) {
        return;
    }
    window.open_dialog(cx, |dialog, window, _| {
        dialog
            .title(crate::i18n::tr("Running without administrator privileges"))
            .border_1()
            .border_color(rgb(0xfbbf24))
            .width(px(560.).min(window.viewport_size().width - px(48.)))
            .overlay_closable(false)
            .child(crate::i18n::tr("PicoForge All is not running as administrator. Some device information may be unavailable."))
            .footer(|_, _, _, cx| vec![
                super::button::standard("continue-without-administrator", cx)
                    .label(crate::i18n::tr("Continue without administrator"))
                    .on_click(|_, window, cx| window.close_dialog(cx)),
                restart_button("administrator-restart-dialog", cx).primary(),
            ])
    });
}

fn restart(window: &mut Window, cx: &mut App) {
    let state = cx.global_mut::<AdministratorState>();
    if !state.required || state.restarting {
        return;
    }
    state.restarting = true;
    window.close_dialog(cx);
    let handle = window.window_handle();
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        let result = cx
            .background_executor()
            .spawn(async { crate::elevation::restart_as_administrator() })
            .await;
        let _ = cx.update(|cx| {
            if matches!(result, Ok(RestartOutcome::Launched)) {
                cx.quit();
                return;
            }
            cx.global_mut::<AdministratorState>().restarting = false;
            if let Err(error) = result {
                log::error!("Administrator restart failed: {error}");
                let _ = handle.update(cx, |_, window, cx| {
                    window.push_notification(
                        crate::i18n::tr("Could not restart as administrator. Please try again."),
                        cx,
                    );
                });
            }
            cx.refresh_windows();
        });
    })
    .detach();
}
