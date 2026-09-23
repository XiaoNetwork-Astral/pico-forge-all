use super::*;
use std::{cell::Cell, collections::VecDeque};

const SERIAL: &str = "432D921975CCC729";

fn absent(serial: &str) -> Result<String, PicotoolError> {
    let suffix = if serial.is_empty() {
        String::new()
    } else {
        format!(" with serial number {serial}")
    };
    Err(PicotoolError::Exit {
        code: -4,
        output: format!("No accessible RP-series devices in BOOTSEL mode were found{suffix}.\r\n"),
    })
}

fn ready(serial: &str) -> Result<String, PicotoolError> {
    Ok(format!(
        "Device Information\n    type: RP2350\n    chipid: 0x{serial}\n"
    ))
}

fn probe(output: Result<String, PicotoolError>) -> Result<Vec<String>, String> {
    parse_bootsel(output, SERIAL, || panic!("unexpected OTP read"))
}

#[test]
fn absence_is_a_device_state_before_user_error_formatting() {
    assert!(probe(absent(SERIAL)).unwrap().is_empty());
    assert!(
        parse_bootsel(absent(""), "", || panic!())
            .unwrap()
            .is_empty()
    );
    assert!(probe(absent("E830F26C33DD8993")).is_err());
    assert_eq!(probe(ready(SERIAL)).unwrap(), [SERIAL]);
}

#[test]
fn update_request_runs_once_and_waits_through_usb_reenumeration() {
    let mut replies = VecDeque::from([
        absent(SERIAL),
        absent(SERIAL),
        absent(SERIAL),
        ready(SERIAL),
    ]);
    let requests = Cell::new(0);
    enter_update_mode(
        SERIAL,
        || probe(replies.pop_front().expect("unexpected probe")),
        || {
            requests.set(requests.get() + 1);
            Ok(())
        },
        Duration::from_secs(1),
        Duration::ZERO,
    )
    .unwrap();
    assert_eq!(requests.get(), 1);
    assert!(replies.is_empty());

    enter_update_mode(
        SERIAL,
        || probe(ready(SERIAL)),
        || panic!("already in update mode"),
        Duration::ZERO,
        Duration::ZERO,
    )
    .unwrap();
}

#[test]
fn access_failures_and_unknown_process_errors_do_not_request_a_reboot() {
    for error in [
        PicotoolError::Exit {
            code: -4,
            output: format!(
                "No accessible RP-series devices in BOOTSEL mode were found with serial number {SERIAL}.\nThe device could not be opened: LIBUSB_ERROR_ACCESS\n"
            ),
        },
        PicotoolError::Exit {
            code: 1,
            output: "ERROR: USB connection lost".into(),
        },
        PicotoolError::Execution("picotool not found".into()),
    ] {
        let mut output = Some(Err(error));
        assert!(
            enter_update_mode(
                SERIAL,
                || probe(output.take().unwrap()),
                || panic!("must not reboot on a tool or driver error"),
                Duration::ZERO,
                Duration::ZERO
            )
            .is_err()
        );
    }
}

#[test]
fn rejected_confirmation_and_transition_timeout_are_reported() {
    let mut replies = VecDeque::from([absent(SERIAL)]);
    let error = enter_update_mode(
        SERIAL,
        || probe(replies.pop_front().unwrap()),
        || Err("Device declined the operation (6985)".into()),
        Duration::ZERO,
        Duration::ZERO,
    )
    .unwrap_err();
    assert!(error.contains("6985"));
    let error = enter_update_mode(
        SERIAL,
        || probe(absent(SERIAL)),
        || Ok(()),
        Duration::ZERO,
        Duration::ZERO,
    )
    .unwrap_err();
    assert_eq!(error, "The selected board did not enter update mode.");
}

#[test]
fn recovery_metadata_uses_otp_identity_and_rejects_another_board() {
    let invalid_metadata = || {
        Err(PicotoolError::Exit {
            code: 1,
            output: "ERROR: Block loop is not valid\n".into(),
        })
    };
    assert_eq!(
        parse_bootsel(invalid_metadata(), SERIAL, || Ok(SERIAL.into())).unwrap(),
        [SERIAL]
    );
    assert!(parse_bootsel(invalid_metadata(), SERIAL, || Ok("E830F26C33DD8993".into())).is_err());
    assert!(parse_bootsel(invalid_metadata(), SERIAL, || Err("OTP read failed".into())).is_err());
}

#[test]
fn probe_rejects_wrong_chips_missing_metadata_and_other_serials() {
    assert!(probe(ready("E830F26C33DD8993")).is_err());
    assert!(probe(Ok(format!("chipid: {SERIAL}\n"))).is_err());
    assert!(probe(Ok(format!("type: RP2040\nchipid: {SERIAL}\n"))).is_err());
    assert!(probe(Ok("type: RP2350\nchipid: not-a-serial\n".into())).is_err());
}

#[test]
#[ignore = "read-only probe; requires PICOTOOL and the selected board in normal mode"]
fn native_normal_mode_probe() {
    let serial = std::env::var("PICOFORGE_TEST_SERIAL").unwrap();
    assert!(serial_valid(&serial));
    let _guard = super::super::transport::pcsc::lock_device().unwrap();
    assert_eq!(
        normal_cards()
            .unwrap()
            .iter()
            .filter(|(s, _)| s == &serial)
            .count(),
        1
    );
    let (log, _) = std::sync::mpsc::channel();
    let worker = Worker {
        tool: std::env::var("PICOTOOL").unwrap(),
        serial,
        log,
    };
    assert!(worker.bootsel().unwrap().is_empty());
}

#[test]
#[ignore = "requires the selected board and one button confirmation; reboots without flashing"]
fn native_update_mode_roundtrip() {
    let serial = std::env::var("PICOFORGE_TEST_SERIAL").unwrap();
    assert!(serial_valid(&serial));
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let reader = std::thread::spawn(move || {
        for line in rx {
            if !line.contains("] [picotool]") {
                println!("{line}");
            }
        }
    });
    for action in ["bootsel", "reboot"] {
        run(
            Request {
                action: action.into(),
                serial: serial.clone(),
                ..Default::default()
            },
            tx.clone(),
        )
        .unwrap();
    }
    drop(tx);
    reader.join().unwrap();
}
