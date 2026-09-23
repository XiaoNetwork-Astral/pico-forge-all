use crate::hal::firmware::{Assessment, ImageInfo, Request};

#[derive(Default)]
pub(super) struct FirmwareSelection {
    pub path: String,
    pub image: Option<ImageInfo>,
    pub assessment: Option<Assessment>,
}
impl FirmwareSelection {
    pub fn file_changed(&mut self, path: &str) {
        if self.path != path {
            *self = Self {
                path: path.into(),
                ..Default::default()
            };
        }
    }

    pub fn device_changed(&mut self, serial: &str) {
        if self.assessment.as_ref().is_some_and(|a| a.serial != serial) {
            self.assessment = None;
        }
    }

    pub fn inspected(&mut self, path: String, image: ImageInfo, assessment: Option<Assessment>) {
        *self = Self {
            path,
            image: Some(image),
            assessment,
        };
    }
}

/// A compatibility result may continue only the flash explicitly confirmed by
/// the user. Local inspection and signing never produce a write request.
pub(super) fn confirmed_flash(request: &Request, assessment: &Assessment) -> Option<Request> {
    if request.action != "check"
        || request.phrase != format!("FLASH {}", request.serial)
        || request.serial != assessment.serial
        || !assessment.allowed
    {
        return None;
    }
    Some(Request {
        action: "flash".into(),
        review: Some(serde_json::to_value(assessment).unwrap()),
        ..request.clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn image(signed: bool, nuke: bool) -> ImageInfo {
        ImageInfo {
            signed,
            nuke,
            fingerprint: signed.then(|| "test-key".into()),
            hash: "digest".into(),
        }
    }

    #[test]
    fn signing_result_survives_the_input_change_event() {
        for nuke in [false, true] {
            let mut selection = FirmwareSelection::default();
            selection.inspected("unsigned.uf2".into(), image(false, nuke), None);
            selection.inspected("signed.uf2".into(), image(true, nuke), None);
            selection.file_changed("signed.uf2"); // deferred programmatic InputEvent::Change
            assert!(selection.image.as_ref().unwrap().signed);
            selection.device_changed("432D921975CCC729");
            assert!(selection.image.is_some());
            selection.file_changed("different.uf2");
            assert!(selection.image.is_none());
            assert!(selection.assessment.is_none());
        }
    }

    #[test]
    fn only_explicit_flash_confirmation_can_continue_after_compatibility_check() {
        for nuke in [false, true] {
            let mut assessment = Assessment {
                image: image(true, nuke),
                serial: "432D921975CCC729".into(),
                secure_boot: true,
                board_key: Some("test-key".into()),
                allowed: true,
                mismatch: false,
            };
            let mut request = Request {
                serial: assessment.serial.clone(),
                ..Default::default()
            };
            for action in ["image", "inspect", "sign", "check"] {
                request.action = action.into();
                assert!(confirmed_flash(&request, &assessment).is_none());
            }
            request.phrase = format!("FLASH {}", request.serial);
            let write = confirmed_flash(&request, &assessment).unwrap();
            assert_eq!(write.action, "flash");
            assert!(write.review.is_some());
            request.action = "inspect".into();
            assert!(confirmed_flash(&request, &assessment).is_none());
            request.action = "check".into();
            assessment.allowed = false;
            assert!(confirmed_flash(&request, &assessment).is_none());
            assessment.allowed = true;
            assessment.serial = "E830F26C33DD8993".into();
            assert!(confirmed_flash(&request, &assessment).is_none());
        }
    }
}
