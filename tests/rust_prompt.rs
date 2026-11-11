use anyhow::Result;
use huw::{
    browser::{Browser, Config},
    dom,
    transport::{Output, Runner},
};
use std::cell::RefCell;
struct Editor {
    html: RefCell<String>,
    filled: RefCell<Option<String>>,
    mode: u8,
}
impl Runner for Editor {
    fn run(&self, argv: &[String], _: f64) -> Result<Output> {
        let at = argv.iter().position(|s| s == "--").unwrap();
        let a = &argv[at + 1..];
        let mut out = String::new();
        if a[0] == "get" {
            out = match a[1].as_str() {
                "url" => "https://higgsfield.ai".into(),
                "count" => "1".into(),
                "html" => format!(
                    "<div role='textbox' contenteditable='true'>{}</div><span role='switch' aria-label='Unlimited mode' aria-checked='true'></span><button data-model-select-trigger='true'>Seedance</button><button type='submit'>Generate Unlimited</button>",
                    self.html.borrow()
                ),
                _ => panic!(),
            };
        }
        if a == ["press", "Backspace"] && self.mode != 1 {
            *self.html.borrow_mut() = "<p><br></p>".into();
        }
        if a[0] == "fill" {
            *self.filled.borrow_mut() = Some(a[2].clone());
            let text = if self.mode == 2 { "wrong" } else { &a[2] };
            let escaped = text
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            *self.html.borrow_mut() = match self.mode {
                3 => escaped.replace('\n', "<br>"),
                4 => escaped
                    .split('\n')
                    .map(|s| format!("<p>{}</p>", if s.is_empty() { "<br>" } else { s }))
                    .collect(),
                _ => escaped,
            };
        }
        Ok(Output {
            success: true,
            stdout: out,
        })
    }
}
fn config() -> Config {
    serde_json::from_value(serde_json::json!({"command":["browser"],"browser":"x"})).unwrap()
}
#[test]
fn whitespace_bom_roundtrip() {
    let expected = " \tA paper crane.\n\nSoft light.\t \n";
    for mode in [0, 3, 4] {
        let e = Editor {
            html: RefCell::new("old".into()),
            filled: RefCell::new(None),
            mode,
        };
        let input = format!("\u{feff}{}", expected.replace('\n', "\r\n"));
        let state = Browser {
            config: config(),
            runner: &e,
        }
        .replace_prompt(&input)
        .unwrap();
        assert_eq!(state["prompt_sha256"], dom::digest(expected));
        assert_eq!(e.filled.borrow().as_deref(), Some(expected));
    }
}
#[test]
fn failed_clear_never_fills() {
    let e = Editor {
        html: RefCell::new("old".into()),
        filled: RefCell::new(None),
        mode: 1,
    };
    assert!(
        Browser {
            config: config(),
            runner: &e
        }
        .replace_prompt("new")
        .is_err()
    );
    assert!(e.filled.borrow().is_none());
}
#[test]
fn wrong_readback_stops() {
    let e = Editor {
        html: RefCell::new("old".into()),
        filled: RefCell::new(None),
        mode: 2,
    };
    assert!(
        Browser {
            config: config(),
            runner: &e
        }
        .replace_prompt("new")
        .is_err()
    );
}
#[test]
fn empty_before_action() {
    let e = Editor {
        html: RefCell::new("old".into()),
        filled: RefCell::new(None),
        mode: 0,
    };
    assert!(
        Browser {
            config: config(),
            runner: &e
        }
        .replace_prompt(" \n\t")
        .is_err()
    );
    assert_eq!(*e.html.borrow(), "old");
}
