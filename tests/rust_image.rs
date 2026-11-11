use huw::{dom, image};
use serde_json::{Value, json};
const FORM: &str = r#"<button data-kind='model'>Seedream 5.0 Pro</button><button aria-label='Resolution'>1K</button><button aria-label='Aspect ratio'>9:16</button><span data-slot='stepper-value'>1/4</span><div role='textbox' contenteditable='true'>test prompt</div><button data-tour-anchor='tour-image-reference-remove' data-media-id='fixture-ref'></button><span role='switch' aria-label='Unlimited mode' aria-checked='true'></span><button type='submit'>Unlimited</button>"#;
fn card(status: &str, nested: bool, loaded: bool) -> Value {
    json!({"status":status,"nested":nested,"image_loaded":loaded,"kind":"image"})
}
#[test]
fn form() {
    let s = dom::form(FORM, true).unwrap();
    assert_eq!(
        image::expected(s.clone(), &s).unwrap()["unlimited_verified"],
        true
    );
    assert_eq!(s["reference_count"], 1);
    assert_eq!(s["settings_complete"], true);
}
#[test]
fn off_credit_disabled() {
    for html in [
        FORM.replace("aria-checked='true'", "aria-checked='false'"),
        FORM.replace(">Unlimited<", ">GENERATE 1.25<"),
        FORM.replace("type='submit'", "type='submit' disabled"),
    ] {
        assert_eq!(dom::form(&html, true).unwrap()["unlimited_verified"], false);
    }
}
#[test]
fn prepared_state_changes() {
    let s = dom::form(FORM, true).unwrap();
    for key in [
        "model",
        "resolution",
        "ratio",
        "batch",
        "reference_ids",
        "prompt_sha256",
    ] {
        let mut changed = s.clone();
        changed[key] = json!("changed");
        assert!(image::expected(changed, &s).is_err());
    }
}
#[test]
fn missing_control() {
    assert!(dom::form(&FORM.replace("aria-label='Resolution'", ""), true).is_err());
}
#[test]
fn nested_pending() {
    assert_eq!(
        image::classify(
            &json!([card("queued", false, false), card("queued", true, false)]),
            "asset"
        )["state"],
        "pending"
    );
}
#[test]
fn conflicts_unknown() {
    for rows in [
        json!([]),
        json!([
            card("completed", false, true),
            card("completed", false, true)
        ]),
        json!([card("completed", false, true), card("queued", true, false)]),
    ] {
        assert_eq!(image::classify(&rows, "asset")["state"], "unknown");
    }
}
#[test]
fn loaded_result_required() {
    assert_eq!(
        image::classify(&json!([card("completed", false, false)]), "a")["state"],
        "unknown"
    );
    assert_eq!(
        image::classify(&json!([card("completed", false, true)]), "a")["state"],
        "completed"
    );
    let mut wrong = card("completed", false, true);
    wrong["kind"] = json!("video");
    assert_eq!(image::classify(&json!([wrong]), "a")["state"], "unknown");
}
#[test]
fn failure() {
    assert_eq!(
        image::classify(&json!([card("failed", false, false)]), "a")["state"],
        "failed"
    );
}
#[test]
fn enhancement_is_not_audio() {
    let html = FORM
        .replace("data-kind='model'", "data-model-select-trigger='true'")
        .replace(">Unlimited<", ">Generate Unlimited<")
        + "<label>On<input type='checkbox' name='enhancePrompt' checked></label>";
    let s = dom::form(&html, false).unwrap();
    assert!(
        s["missing_settings"]
            .as_array()
            .unwrap()
            .contains(&json!("Audio"))
    );
}
