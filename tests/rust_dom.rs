use anyhow::Result;
use huw::{
    browser::{Browser, Config},
    dom,
    transport::{Output, Runner},
};
use std::cell::RefCell;
fn form() -> String {
    r#"<div role="textbox" contenteditable="true" data-lexical-editor="true">A <span>paper</span> bird</div><button data-model-select-trigger="true">Seedance Unlimited</button><span role="switch" aria-label="Unlimited mode" aria-checked="true"></span><button type="submit">Generate Unlimited</button>"#.into()
}
#[test]
fn verified() {
    assert_eq!(
        dom::form(&form(), false).unwrap()["unlimited_verified"],
        true
    );
}
#[test]
fn badge_not_toggle() {
    assert_eq!(
        dom::form(
            &form().replace("aria-checked=\"true\"", "aria-checked=\"false\""),
            false
        )
        .unwrap()["unlimited_verified"],
        false
    );
}
#[test]
fn credit_labels() {
    for label in [
        "Generate 90",
        "Generate Unlimited 90",
        "Unlimited loading",
        "Generate",
    ] {
        assert_eq!(
            dom::form(&form().replace("Generate Unlimited", label), false).unwrap()["unlimited_verified"],
            false
        );
    }
}
#[test]
fn disabled() {
    for attr in ["disabled", "aria-disabled=\"true\""] {
        for target in ["type=\"submit\"", "role=\"switch\""] {
            assert_eq!(
                dom::form(&form().replace(target, &format!("{target} {attr}")), false).unwrap()["unlimited_verified"],
                false
            );
        }
    }
}
#[test]
fn korean() {
    assert_eq!(
        dom::form(
            &form()
                .replace("Generate Unlimited", "생성하기 무제한")
                .replace("Unlimited mode", "Unlimited 모드"),
            false
        )
        .unwrap()["unlimited_verified"],
        true
    );
}
#[test]
fn duplicate() {
    assert!(
        dom::form(
            &(form() + "<span role='switch' aria-label='Unlimited mode'></span>"),
            false
        )
        .is_err()
    );
}
#[test]
fn inline_space() {
    assert_eq!(
        dom::form(&form(), false).unwrap()["prompt_sha256"],
        dom::digest("A paper bird")
    );
}
#[test]
fn prompt_boundaries() {
    for (markup, expected) in [
        ("<p>first</p><p><br></p><p>last</p>", "first\n\nlast"),
        ("<div>first</div><div>last</div>", "first\nlast"),
        ("<p>first<br>last</p>", "first\nlast"),
        ("<p> first\t </p><p><br></p>", " first\t \n"),
        ("<p><br></p>", ""),
        ("<br>", ""),
        ("first<br><br>last", "first\n\nlast"),
    ] {
        assert_eq!(
            dom::form(&form().replace("A <span>paper</span> bird", markup), false).unwrap()["prompt_sha256"],
            dom::digest(expected),
            "{markup}"
        );
    }
}
#[test]
fn privacy() {
    let out=dom::form(&(form()+"<div data-tour-anchor='tour-video-reference'><img src='https://private.example/image'></div>"),false).unwrap().to_string();
    assert!(!out.contains("private.example"));
    assert!(!out.contains("A paper bird"));
}
#[test]
fn settings_incomplete() {
    let s = dom::form(&form(), false).unwrap();
    assert_eq!(s["settings_complete"], false);
    assert_eq!(s["missing_settings"].as_array().unwrap().len(), 5);
}
#[test]
fn settings_current_dom() {
    let mut s = form();
    for (k, v) in [
        ("Duration", "6s"),
        ("Audio", "On"),
        ("Ratio", "9:16"),
        ("Quality", "480p"),
        ("Bitrate", "High"),
    ] {
        s += &format!("<button aria-label='{k}'>{v}</button>");
    }
    assert_eq!(dom::form(&s, false).unwrap()["settings_complete"], true);
}
fn asset(id: &str, status: &str, video: bool) -> String {
    format!(
        "<div data-asset-id='{id}' data-job-status='{status}'>{}</div>",
        if video { "<video></video>" } else { "<img>" }
    )
}
#[test]
fn completed() {
    assert_eq!(
        dom::history(&asset("a", "completed", true), "a")["state"],
        "completed"
    );
}
#[test]
fn missing_duplicate() {
    for s in [
        asset("b", "completed", true),
        asset("a", "completed", true) + &asset("a", "completed", true),
    ] {
        assert_eq!(dom::history(&s, "a")["state"], "unknown");
    }
}
#[test]
fn order() {
    assert_eq!(
        dom::history(
            &(asset("b", "completed", true) + &asset("a", "processing", false)),
            "a"
        )["state"],
        "pending"
    );
}
#[test]
fn no_video() {
    assert_eq!(
        dom::history(&asset("a", "completed", false), "a")["state"],
        "unknown"
    );
}
#[test]
fn enums() {
    for s in [
        "pending",
        "queued",
        "processing",
        "generating",
        "in_progress",
    ] {
        assert_eq!(dom::history(&asset("a", s, false), "a")["state"], "pending");
    }
    for s in ["failed", "error"] {
        assert_eq!(dom::history(&asset("a", s, false), "a")["state"], "failed");
    }
    assert_eq!(
        dom::history(&asset("a", "new_status", true), "a")["state"],
        "unknown"
    );
}
#[test]
fn nested_wrappers() {
    let s = format!(
        "<div data-asset-id='a' data-job-status='queued'>{}</div>",
        asset("a", "queued", false)
    );
    assert_eq!(dom::history(&s, "a")["state"], "pending");
    assert_eq!(
        dom::history(&s.replacen("queued", "completed", 1), "a")["state"],
        "unknown"
    );
}
struct Fake {
    outputs: RefCell<Vec<String>>,
    calls: RefCell<Vec<Vec<String>>>,
}
impl Runner for Fake {
    fn run(&self, a: &[String], _: f64) -> Result<Output> {
        self.calls.borrow_mut().push(a.to_vec());
        Ok(Output {
            success: true,
            stdout: self.outputs.borrow_mut().remove(0),
        })
    }
}
fn config() -> Config {
    serde_json::from_value(serde_json::json!({"command":["terminal-browser"],"browser":"session"}))
        .unwrap()
}
#[test]
fn shell_free_argument() {
    let f = Fake {
        outputs: RefCell::new(vec!["ok".into()]),
        calls: RefCell::new(vec![]),
    };
    let b = Browser {
        config: config(),
        runner: &f,
    };
    let p = "hello \"quoted\"\n$(whoami); `secret`";
    b.action(&["fill", "selector", p]).unwrap();
    assert_eq!(f.calls.borrow()[0].last().unwrap(), p);
}
#[test]
fn origin_guard() {
    for url in [
        "https://higgsfield.ai.evil.test",
        "http://higgsfield.ai",
        "https://example.org",
    ] {
        let f = Fake {
            outputs: RefCell::new(vec![url.into()]),
            calls: RefCell::new(vec![]),
        };
        assert!(
            Browser {
                config: config(),
                runner: &f
            }
            .check_origin()
            .is_err()
        );
    }
}
#[test]
fn ensure_on_and_off() {
    for off in [false, true] {
        let mut outputs = vec![
            "https://higgsfield.ai".into(),
            "1".into(),
            if off {
                form().replace("aria-checked=\"true\"", "aria-checked=\"false\"")
            } else {
                form()
            },
        ];
        if off {
            outputs.push("clicked".into());
        }
        outputs.extend(["https://higgsfield.ai".into(), "1".into(), form()]);
        let f = Fake {
            outputs: RefCell::new(outputs),
            calls: RefCell::new(vec![]),
        };
        Browser {
            config: config(),
            runner: &f,
        }
        .ensure()
        .unwrap();
        let calls = f.calls.borrow();
        let clicks: Vec<_> = calls
            .iter()
            .filter(|a| a.iter().any(|s| s == "click"))
            .collect();
        assert_eq!(clicks.len(), usize::from(off));
        assert!(!calls.iter().flatten().any(|s| s.contains("type=submit")));
    }
}
