use anyhow::Result;
use huw::{
    browser::{Browser, Config},
    transport::{Output, Runner},
};
use std::cell::RefCell;
#[test]
fn numeric_config_identifiers() {
    let c: Config =
        serde_json::from_value(serde_json::json!({"command":["browser"],"browser":7553,"tab":4}))
            .unwrap();
    assert_eq!(c.browser, "7553");
    assert_eq!(c.tab, "4");
}
fn config() -> Config {
    serde_json::from_value(serde_json::json!({"command":["browser"],"browser":"test"})).unwrap()
}
struct Failure;
impl Runner for Failure {
    fn run(&self, _: &[String], _: f64) -> Result<Output> {
        Ok(Output {
            success: false,
            stdout: "private prompt https://secret.example".into(),
        })
    }
}
#[test]
fn transport_error_does_not_leak() {
    let b = Browser {
        config: config(),
        runner: &Failure,
    };
    let e = b.action(&["get", "html", "main"]).unwrap_err().to_string();
    assert!(!e.contains("private"));
    assert!(!e.contains("secret"));
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
fn explicit_login_distinct_from_drift() {
    for login in [true, false] {
        let f = Sequence(RefCell::new(vec![
            "https://higgsfield.ai".into(),
            "0".into(),
            login.to_string(),
        ]));
        let e = Browser {
            config: config(),
            runner: &f,
        }
        .inspect()
        .unwrap_err()
        .to_string();
        assert_eq!(e.contains("signed_out"), login);
    }
}
