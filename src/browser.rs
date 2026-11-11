use crate::{dom, transport::Runner};
use anyhow::{Result, bail};
use serde::Deserialize;
use serde_json::Value;
#[derive(Deserialize)]
pub struct Config {
    pub command: Vec<String>,
    #[serde(deserialize_with = "identifier")]
    pub browser: String,
    #[serde(default = "tab", deserialize_with = "identifier")]
    pub tab: String,
    #[serde(default = "timeout")]
    pub command_timeout_seconds: f64,
    #[serde(default)]
    pub media_kind: String,
}
fn identifier<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    match Value::deserialize(d)? {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        _ => Err(serde::de::Error::custom(
            "browser/tab must be string or number",
        )),
    }
}
fn tab() -> String {
    "1".into()
}
fn timeout() -> f64 {
    30.0
}
pub const VIDEO: &str = "form[data-tour-anchor=\"tour-video-form\"]";
pub struct Browser<'a> {
    pub config: Config,
    pub runner: &'a dyn Runner,
}
impl Browser<'_> {
    pub fn image(&self) -> bool {
        self.config.media_kind == "image"
    }
    pub fn form(&self) -> &str {
        if self.image() {
            "form#image-form"
        } else {
            VIDEO
        }
    }
    pub fn action(&self, args: &[&str]) -> Result<String> {
        let c = &self.config;
        if c.command.is_empty()
            || c.command.iter().any(String::is_empty)
            || !c.command_timeout_seconds.is_finite()
            || !(1.0..=120.0).contains(&c.command_timeout_seconds)
            || !matches!(c.media_kind.as_str(), "" | "video" | "image")
        {
            bail!("invalid browser config");
        }
        let mut argv = c.command.clone();
        argv.extend(["action", "--browser", &c.browser, "--tab", &c.tab, "--"].map(String::from));
        argv.extend(args.iter().map(|s| s.to_string()));
        let o = self.runner.run(&argv, c.command_timeout_seconds)?;
        if !o.success {
            bail!("browser command failed; inspect session locally");
        }
        Ok(o.stdout)
    }
    pub fn check_origin(&self) -> Result<()> {
        let s = self.action(&["get", "url"])?;
        let u = url::Url::parse(&s)?;
        if u.scheme() != "https"
            || !matches!(u.host_str(), Some("higgsfield.ai" | "www.higgsfield.ai"))
        {
            bail!("expected an HTTPS Higgsfield tab");
        }
        Ok(())
    }
    pub fn inspect(&self) -> Result<Value> {
        self.check_origin()?;
        if self.action(&["get", "count", self.form()])? != "1" {
            let login=self.action(&["eval","Array.from(document.querySelectorAll('button,a')).some(n=>/^(Login|Log in|Sign in|로그인)$/.test(n.innerText.trim()))"])?;
            if login == "true" {
                bail!("signed_out: explicit login control present; no automatic login");
            }
            bail!("selector drift: expected one creation form");
        }
        dom::form(&self.action(&["get", "html", self.form()])?, self.image())
    }
    pub fn ensure(&self) -> Result<Value> {
        let s = self.inspect()?;
        if s["switch_disabled"] == true {
            bail!("Unlimited switch disabled");
        }
        if s["unlimited"] == "false" {
            let sel = format!(
                "{} [role=switch][aria-label='Unlimited mode'],{} [role=switch][aria-label='Unlimited 모드']",
                self.form(),
                self.form()
            );
            self.action(&["click", &sel])?;
        }
        let s = self.inspect()?;
        if s["unlimited_verified"] != true {
            bail!("Unlimited switch and submit label not verified");
        }
        Ok(s)
    }
    pub fn replace_prompt(&self, value: &str) -> Result<Value> {
        let value = dom::normalize(value.strip_prefix('\u{feff}').unwrap_or(value));
        if value.trim().is_empty() {
            bail!("prompt file is empty");
        }
        self.inspect()?;
        let selector = format!("{} [role=textbox][contenteditable=true]", self.form());
        self.action(&["click", &selector])?;
        self.action(&["press", "Control+a"])?;
        self.action(&["press", "Backspace"])?;
        if self.inspect()?["prompt_length"] != 0 {
            bail!("prompt did not clear; no text was appended");
        }
        self.action(&["fill", &selector, &value])?;
        let state = self.inspect()?;
        if state["prompt_sha256"] != dom::digest(&value) {
            bail!("prompt readback differs; inspect editor before generation");
        }
        Ok(state)
    }
    pub fn observe(&self, aid: &str) -> Result<Value> {
        self.check_origin()?;
        let aid = crate::monitor::asset(aid)?;
        if self.image() {
            let js = format!(
                r#"(() => {{const id={}; const sel='[data-asset-id="'+id+'"]';return Array.from(document.querySelectorAll(sel)).map(n=>({{nested:!!n.parentElement?.closest(sel),status:n.getAttribute('data-job-status'),kind:n.getAttribute('data-tour-asset-kind'),image_loaded:Array.from(n.querySelectorAll('img[alt="image generation"]')).some(i=>i.complete&&i.naturalWidth>0&&i.naturalHeight>0)}}));}})()"#,
                serde_json::to_string(&aid)?
            );
            let cards: Value = serde_json::from_str(&self.action(&["eval", &js])?)?;
            return Ok(crate::image::classify(&cards, &aid));
        }
        Ok(dom::history(&self.action(&["get", "html", "main"])?, &aid))
    }
}
