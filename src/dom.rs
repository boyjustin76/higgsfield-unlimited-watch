use anyhow::{Result, bail};
use scraper::{ElementRef, Html, Selector};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
pub fn digest(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}
pub fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}
pub fn text(n: ElementRef<'_>) -> String {
    n.text().collect::<String>().trim().into()
}
pub fn disabled(n: ElementRef<'_>) -> bool {
    n.attr("disabled").is_some()
        || n.attr("data-disabled").is_some()
        || n.attr("aria-disabled") == Some("true")
}
pub fn one<'a>(d: &'a Html, css: &str) -> Result<ElementRef<'a>> {
    let sel = Selector::parse(css).map_err(|_| anyhow::anyhow!("invalid internal selector"))?;
    let nodes: Vec<_> = d.select(&sel).collect();
    if nodes.len() != 1 {
        bail!("selector drift: expected 1, found {}", nodes.len());
    }
    Ok(nodes[0])
}
fn block(n: ElementRef<'_>) -> bool {
    matches!(
        n.value().name(),
        "p" | "div" | "li" | "pre" | "blockquote" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
    )
}
pub fn prompt(n: ElementRef<'_>) -> String {
    if n.value().name() == "br" {
        return "\n".into();
    }
    let children: Vec<_> = n.children().collect();
    if (block(n) || n.attr("role") == Some("textbox"))
        && children.len() == 1
        && ElementRef::wrap(children[0]).is_some_and(|e| e.value().name() == "br")
    {
        return String::new();
    }
    let mut out = String::new();
    let mut prev = false;
    for (i, node) in children.iter().enumerate() {
        let e = ElementRef::wrap(*node);
        let current = e.is_some_and(block);
        if i > 0 && (prev || current) {
            out.push('\n');
        }
        if let Some(e) = e {
            out.push_str(&prompt(e));
        } else if let Some(t) = node.value().as_text() {
            out.push_str(t);
        }
        prev = current;
    }
    normalize(&out)
}
pub fn form(html: &str, image: bool) -> Result<Value> {
    let d = Html::parse_fragment(html);
    let p = one(&d, "[role=textbox][contenteditable=true]")?;
    let sw = one(
        &d,
        "[role=switch][aria-label='Unlimited mode'],[role=switch][aria-label='Unlimited 모드']",
    )?;
    let sub = one(&d, "button[type=submit]")?;
    let model = one(
        &d,
        if image {
            "[data-kind=model]"
        } else {
            "[data-model-select-trigger=true]"
        },
    )?;
    let mut settings = Map::new();
    for n in
        d.select(&Selector::parse("[data-form-kit-setting-button],button[aria-label]").unwrap())
    {
        let label = n.attr("aria-label").unwrap_or("unknown");
        if n.attr("data-form-kit-setting-button").is_some()
            || matches!(
                label,
                "Duration"
                    | "Ratio"
                    | "Quality"
                    | "Bitrate"
                    | "Resolution"
                    | "Aspect ratio"
                    | "Audio"
            )
        {
            if settings.insert(label.into(), json!(text(n))).is_some() {
                bail!("ambiguous setting key");
            }
        }
    }
    let refs: Vec<_> = d
        .select(
            &Selector::parse(if image {
                "[data-tour-anchor=tour-image-reference-remove]"
            } else {
                "[data-tour-anchor=tour-video-reference] img"
            })
            .unwrap(),
        )
        .collect();
    let label = text(sub);
    let compact: String = label.chars().filter(|c| !c.is_whitespace()).collect();
    let recognized = if image {
        matches!(compact.as_str(), "Unlimited" | "UNLIMITED")
    } else {
        matches!(compact.as_str(), "GenerateUnlimited" | "생성하기무제한")
    };
    let expected = if image {
        vec!["Resolution", "Aspect ratio"]
    } else {
        vec!["Duration", "Ratio", "Quality", "Bitrate", "Audio"]
    };
    let missing: Vec<_> = expected
        .into_iter()
        .filter(|x| !settings.contains_key(*x))
        .collect();
    let ptext = prompt(p);
    let mut out = json!({"unlimited":sw.attr("aria-checked"),"switch_disabled":disabled(sw),"submit_label":label,"submit_disabled":disabled(sub),"unlimited_verified":sw.attr("aria-checked")==Some("true")&&!disabled(sw)&&recognized&&!disabled(sub),"model":text(model),"settings":settings,"settings_complete":missing.is_empty(),"missing_settings":missing,"prompt_sha256":digest(&ptext),"prompt_length":ptext.chars().count(),"reference_count":refs.len()});
    if image {
        out["resolution"] = json!(text(one(&d, "button[aria-label=Resolution]")?));
        out["ratio"] = json!(text(one(&d, "button[aria-label='Aspect ratio']")?));
        out["batch"] = json!(text(one(&d, "[data-slot=stepper-value]")?));
        out["reference_ids"] = json!(
            refs.iter()
                .map(|n| n.attr("data-media-id"))
                .collect::<Vec<_>>()
        );
    }
    Ok(out)
}
pub fn history(html: &str, aid: &str) -> Value {
    let d = Html::parse_fragment(html);
    let sel = Selector::parse("[data-asset-id]").unwrap();
    let nodes: Vec<_> = d
        .select(&sel)
        .filter(|n| n.attr("data-asset-id") == Some(aid))
        .collect();
    let unknown =
        json!({"state":"unknown","reason":"asset_missing_or_ambiguous","matches":nodes.len()});
    let roots: Vec<_> = nodes
        .iter()
        .filter(|n| {
            !n.ancestors()
                .filter_map(ElementRef::wrap)
                .any(|a| a.attr("data-asset-id") == Some(aid))
        })
        .collect();
    if roots.len() != 1 {
        return unknown;
    }
    let raw = roots[0].attr("data-job-status");
    if nodes.iter().any(|n| n.attr("data-job-status") != raw) {
        return unknown;
    }
    let video = roots[0]
        .select(&Selector::parse("video").unwrap())
        .next()
        .is_some();
    let state = match raw {
        Some("pending" | "queued" | "processing" | "generating" | "in_progress") => "pending",
        Some("failed" | "error") => "failed",
        Some("completed") if video => "completed",
        _ => "unknown",
    };
    json!({"state":state,"asset_id":aid,"dom_status":raw,"video_present":video,"reason":"dom_status"})
}
