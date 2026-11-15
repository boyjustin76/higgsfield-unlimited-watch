use anyhow::{Result, bail};
use serde_json::{Value, json};
pub fn expected(state: Value, wanted: &Value) -> Result<Value> {
    for key in [
        "model",
        "resolution",
        "ratio",
        "batch",
        "reference_ids",
        "prompt_sha256",
    ] {
        if wanted.get(key).is_none() || state.get(key) != wanted.get(key) {
            bail!("prepared form changed: {key}");
        }
    }
    if state["unlimited_verified"] != true || state["prompt_length"].as_u64().unwrap_or(0) == 0 {
        bail!("prepared image form not ready");
    }
    Ok(state)
}
pub fn classify(cards: &Value, aid: &str) -> Value {
    let Some(cards) = cards.as_array() else {
        return json!({"state":"unknown","reason":"invalid_observation"});
    };
    let unknown =
        json!({"state":"unknown","reason":"asset_missing_or_ambiguous","matches":cards.len()});
    if cards.is_empty() || cards.iter().filter(|c| c["nested"] == false).count() != 1 {
        return unknown;
    }
    let status = &cards[0]["status"];
    if cards.iter().any(|c| &c["status"] != status)
        || !cards.iter().any(|c| c["kind"] == "image")
        || cards
            .iter()
            .any(|c| !c["kind"].is_null() && c["kind"] != "image")
    {
        return unknown;
    }
    let loaded = cards.iter().any(|c| c["image_loaded"] == true);
    let state = match status.as_str() {
        Some("pending" | "queued" | "processing" | "generating" | "in_progress") => "pending",
        Some("failed" | "error") => "failed",
        Some("completed") if loaded => "completed",
        _ => "unknown",
    };
    json!({"state":state,"asset_id":aid,"dom_status":status,"image_present":loaded,"reason":"image_dom_status"})
}
