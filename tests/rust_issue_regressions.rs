use anyhow::Result;
use huw::{
    browser::{Browser, Config},
    dom,
    transport::{Output, Runner},
};
use std::cell::RefCell;

fn form() -> String {
    r#"<div role="textbox" contenteditable="true" data-lexical-editor="true"></div><button data-model-select-trigger="true">Example model</button><span role="switch" aria-label="Unlimited mode" aria-checked="true"></span><button type="submit">Generate Unlimited</button>"#.into()
}

#[test]
fn empty_settings_are_missing() {
    let state = dom::form(&(form() + "<button aria-label='Duration'></button>"), false).unwrap();
    assert!(
        state["missing_settings"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("Duration"))
    );
}

#[test]
fn nested_completed_and_conflicting_or_separate_cards() {
    let card = "<div data-asset-id='a' data-job-status='completed'><video></video></div>";
    assert_eq!(
        dom::history(
            &format!("<div data-asset-id='a' data-job-status='completed'>{card}</div>"),
            "a"
        )["state"],
        "completed"
    );
    assert_eq!(
        dom::history(&format!("{card}{card}"), "a")["state"],
        "unknown"
    );
    assert_eq!(
        dom::history(
            &format!("<div data-asset-id='a' data-job-status='queued'>{card}</div>"),
            "a"
        )["state"],
        "unknown"
    );
}

struct Sequence(RefCell<Vec<String>>);
impl Runner for Sequence {
    fn run(&self, _: &[String], _: f64) -> Result<Output> {
        Ok(Output {
            success: true,
            stdout: self.0.borrow_mut().remove(0),
        })
    }
}
#[test]
fn missing_switch_in_existing_form_is_diagnosed() {
    for (ready, login, expected) in [
        ("complete", true, "signed_out"),
        ("loading", false, "page_not_ready"),
        ("complete", false, "missing_control"),
        ("mystery", false, "unknown"),
    ] {
        let f = Sequence(RefCell::new(vec![
            "https://higgsfield.ai/".into(),
            "1".into(),
            form().replace("Unlimited mode", "Changed label"),
            serde_json::json!({"ready":ready,"login":login}).to_string(),
        ]));
        let config: Config =
            serde_json::from_value(serde_json::json!({"command":["browser"],"browser":"test"}))
                .unwrap();
        let error = Browser { config, runner: &f }
            .inspect()
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{error}");
    }
}
