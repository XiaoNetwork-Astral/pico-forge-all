//! Feature switches shown on the page that uses them, including when disabled.
use super::{card::Card, dialog, form::FormErrors};
use crate::ui::models::device::{
    DeviceMethod, DeviceRepo, USB_CAP_FIDO2, USB_CAP_HSM, USB_CAP_OATH, USB_CAP_OPENPGP,
    USB_CAP_OTP, USB_CAP_PIV, USB_CAP_U2F,
};
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::InputState,
    switch::Switch,
    v_flex,
};

pub fn render(device: &Entity<DeviceRepo>, caps: &[u16], cx: &App) -> Option<AnyElement> {
    let repo = device.read(cx);
    let apps = repo.management_apps.as_ref()?;
    let mut rows = v_flex().gap_4();
    let mut visible = false;
    for &cap in caps {
        let supported = apps.usb_supported & cap != 0;
        if !supported && cap != USB_CAP_HSM {
            continue;
        }
        visible = true;
        let (name, description) = match cap {
            USB_CAP_FIDO2 => ("FIDO2", "Passkeys and passwordless sign-in"),
            USB_CAP_U2F => ("U2F", "Legacy two-step security-key sign-in"),
            USB_CAP_OATH => ("OATH", "Verification codes for accounts (TOTP / HOTP)"),
            USB_CAP_OTP => ("OTP", "Button-triggered output from the configured slots"),
            USB_CAP_PIV => ("PIV", "Smart-card keys and certificates"),
            USB_CAP_OPENPGP => ("OpenPGP", "OpenPGP keys and card operations"),
            USB_CAP_HSM => ("HSM", "SmartCard-HSM keys, certificates and objects"),
            _ => continue,
        };
        let device = device.clone();
        rows = rows.child(
            h_flex()
                .justify_between()
                .items_center()
                .gap_4()
                .child(
                    v_flex().child(name).child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(crate::i18n::tr(if supported {
                                description
                            } else {
                                "Update firmware to enable or disable HSM."
                            })),
                    ),
                )
                .child(
                    Switch::new(SharedString::from(format!("feature-{cap}")))
                        .checked(if cap == USB_CAP_HSM && !supported {
                            true
                        } else {
                            apps.usb_enabled & cap != 0
                        })
                        .disabled(!supported || repo.loading || repo.applying_apps)
                        .on_click(move |enabled, window, cx| {
                            open(&device, cap, *enabled, window, cx)
                        }),
                ),
        );
    }
    if !visible {
        return None;
    }
    Some(
        Card::new()
            .title(crate::i18n::tr("Enable feature"))
            .child(rows)
            .into_any_element(),
    )
}

fn open(device: &Entity<DeviceRepo>, cap: u16, enabled: bool, window: &mut Window, cx: &mut App) {
    let repo = device.read(cx);
    let Some(state) = &repo.status else {
        return;
    };
    if repo.applying_apps {
        return;
    }
    let method = state.method.clone();
    let serial = state.info.serial.clone();
    let pin_required = method == DeviceMethod::Fido;
    let pin = cx.new(|cx| InputState::new(window, cx).masked(true));
    let errors = FormErrors::default();
    errors.watch(0, &pin, window, cx);
    let device = device.clone();
    let submit = std::rc::Rc::new({
        let pin = pin.clone();
        let errors = errors.clone();
        move |window: &mut Window, cx: &mut App| {
            let value = pin.read(cx).text().to_string();
            errors.clear();
            if pin_required {
                errors.required(0, "FIDO PIN", &value);
            }
            if !errors.valid(window) {
                return;
            }
            window.close_dialog(cx);
            let status =
                dialog::open_status_dialog(crate::i18n::tr("Applying Configuration"), window, cx);
            let _ = status.update(cx, |d, cx| {
                d.set_loading(super::copy::CONFIRM_ON_DEVICE, cx)
            });
            let method = method.clone();
            let serial = serial.clone();
            device.update(cx, |repo, cx| {
                repo.applying_apps = true;
                cx.notify();
                cx.spawn(async move |device, cx| {
                    let result = cx
                        .background_executor()
                        .spawn(async move {
                            DeviceRepo::set_application_blocking(
                                method,
                                serial,
                                cap,
                                enabled,
                                pin_required.then_some(value),
                            )
                        })
                        .await;
                    let _ = device.update(cx, |repo, cx| {
                        repo.applying_apps = false;
                        match result {
                            Ok(state) => {
                                repo.apply_fresh_state(state, cx);
                                let _ = status.update(cx, |d, cx| {
                                    d.set_success(
                                        crate::i18n::tr("Configuration applied.").into(),
                                        cx,
                                    )
                                });
                            }
                            Err(error) => {
                                let _ =
                                    status.update(cx, |d, cx| d.set_error(error.to_string(), cx));
                            }
                        }
                        cx.notify();
                    });
                })
                .detach();
            });
        }
    });
    window.open_dialog(cx, move |dialog, _, _| {
        let ok = submit.clone();
        let button = submit.clone();
        dialog
            .title(crate::i18n::tr(if enabled {
                "Enable feature"
            } else {
                "Disable feature"
            }))
            .child(crate::i18n::tr(
                "Stored credentials are preserved when a feature is disabled.",
            ))
            .children(pin_required.then(|| errors.field(0, "FIDO PIN", &pin, true)))
            .on_ok(move |_, window, cx| {
                ok(window, cx);
                false
            })
            .footer(move |_, _, _, _| {
                let button = button.clone();
                vec![
                    Button::new("cancel")
                        .label(crate::i18n::tr("Cancel"))
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                    Button::new("apply")
                        .primary()
                        .label(crate::i18n::tr("Apply"))
                        .on_click(move |_, window, cx| button(window, cx)),
                ]
            })
    });
}
