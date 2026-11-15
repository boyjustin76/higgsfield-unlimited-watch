# Setup and commands

Rust is the primary executable. From a repository checkout, run `cargo build --locked --release` then `cargo install --path . --locked` using stable Rust. Run `huw --help` after Cargo's bin directory is on PATH, or use `target/release/huw` (`target/release/huw.exe` on Windows). This does not replace any existing shared installation.

The skill folder alone contains the retained Python implementation. The Python commands below remain valid for video workflows; substitute `huw` for `python skills/higgsfield-unlimited-watch/scripts/huw.py` to use Rust. After installing only the skill, use its actual folder path. Neither `demo` nor `inspect` requires Herdr (inspect still requires terminal-browser). Python non-ASCII output on Windows pipes should use `python -X utf8` ([Python documentation](https://docs.python.org/3/using/cmdline.html#cmdoption-X)).

## Existing browser setup

Python 3.10+ is required; no pip dependencies. In PowerShell:

```powershell
python skills/higgsfield-unlimited-watch/scripts/huw.py demo
wsl.exe -d Ubuntu --exec /home/YOUR_LINUX_USER/.local/bin/terminal-browser ls --all
Copy-Item config.example.json config.local.json
```

Edit `config.local.json`: set the actual Linux binary path, discovered browser ID and intended tab. The `command` array is executable + arguments, not a shell string. Only use trusted config files: they select a local executable. For native Linux, `command` can be `["/home/YOU/.local/bin/terminal-browser"]`; live Linux-host support is not yet validated.

If you installed only the skill folder, create a local config yourself instead of copying the repository example:

```json
{
  "command": ["wsl.exe", "-d", "Ubuntu", "--exec", "/home/YOUR_LINUX_USER/.local/bin/terminal-browser"],
  "browser": "YOUR_DISCOVERED_BROWSER_ID",
  "tab": "1",
  "command_timeout_seconds": 30
}
```

Keep that config and state in your own local working directory, not inside a shared skill installation.

```powershell
python skills/higgsfield-unlimited-watch/scripts/huw.py inspect --config config.local.json
python skills/higgsfield-unlimited-watch/scripts/huw.py ensure-unlimited --config config.local.json
python skills/higgsfield-unlimited-watch/scripts/huw.py set-prompt --config config.local.json --file local/prompt.txt
```

`inspect` is read-only. `ensure-unlimited` may click the toggle; `set-prompt` changes the editor. None submits a generation. Always check full settings and concurrency separately.

## First Windows installation

Check existing installations before adding anything. In PowerShell:

```powershell
wsl --list --verbose
wsl --install -d Ubuntu
winget install --id AmanThanvi.noctty -e
```

Install only missing components. Complete Ubuntu user creation, then in Ubuntu:

```bash
sudo apt-get update
sudo apt-get install -y curl ca-certificates bash libnspr4 libnss3 libgtk-3-0 libasound2t64 libgbm1 fonts-noto-cjk
curl -fsSL https://terminal-browser.sh/install | bash
export PATH="$HOME/.local/bin:$PATH"
```

Persist the PATH export once in `~/.bashrc`. Package names are from the tested Ubuntu setup; use distro equivalents if needed. Do not install into the `docker-desktop` distro. Launch Noctty, run `wsl -d Ubuntu`, then:

```bash
terminal-browser open 'https://higgsfield.ai/ai/video?model=seedance_2_0'
```

Sign in directly. An unsupported-terminal warning from a non-interactive command does not establish a failure in the visible graphical terminal. Check the actual Noctty session separately. Restart the browser after font installation if Korean text remains broken. Windows Chrome and the WSL browser do not automatically share authentication. See [terminal-browser](https://github.com/zenbu-labs/terminal-browser) for current installer support.

## Blank page / TLS diagnosis

1. Check whether the intended browser process/session is running. Then distinguish a page load/TLS error from terminal graphics support; do not reinstall based on a blank display alone.
2. Inspect locally for `ERR_CERT_AUTHORITY_INVALID` or curl error 60. Windows trust does not automatically establish Ubuntu or Chromium trust. Do not publish raw logs, certificates or fingerprints.
3. Use only an administrator-approved CA. Follow [Ubuntu's trust-store instructions](https://ubuntu.com/server/docs/how-to/security/install-a-root-ca-certificate-in-the-trust-store/): approved PEM certificate with .crt extension, system CA location and update-ca-certificates. Never disable certificate verification.
4. Chromium also uses NSS. [Official Chromium documentation](https://chromium.googlesource.com/chromium/src/+/main/docs/linux/cert_management.md) specifies `$HOME/.local/share/pki/nssdb` since M146, but uses an existing `$HOME/.pki/nssdb` instead. Determine the actual browser user's DB before applying the documented certutil instructions; do not create/import into a guessed DB.
5. After approved trust changes, restart the affected browser and confirm HTTPS page loading. A running process or successful curl alone is insufficient. Graphics warnings from a headless command are a separate diagnostic.

## Form diagnostics and new tabs

The helper reports signed_out only with explicit login evidence during a failed control check; loading becomes page_not_ready. A complete page with absent controls is missing_control, ambiguous controls selector_drift, insufficient evidence unknown. These codes do not infer paid entitlement. No automatic login or cookie transfer is provided.

Inspect each new tab/navigation/settings change independently. Recheck switch, submit label/enabled state and all intended settings immediately before a separately authorized submission. Empty/unknown settings remain missing, even if `unlimited_verified` is true. See [official availability guidance](https://higgsfield.ai/creator-hub/help-center/credits/what-are-unlimited-models-and-which-plans-include-them); no account eligibility is assumed.

## Herdr background monitor

On herdr-core desktop, use `hc`, not the older `herdr` multiplexer. Typical Windows location:
`$env:LOCALAPPDATA\Programs\herdr-core\hc.exe`.

```powershell
$hc = Join-Path $env:LOCALAPPDATA 'Programs\herdr-core\hc.exe'
& $hc ls
```

Select your actual receiving agent terminal ID. Do not copy another machine's ID. Start from the checkout directory, substituting UUID and terminal ID:

```powershell
& $hc run --bg --no-notify --name higgs-watch python skills/higgsfield-unlimited-watch/scripts/huw.py watch --config config.local.json --asset-id RESULT_UUID --state-dir local/job-001 --hc $hc --notify-terminal AGENT_TERMINAL_ID --notify-app copilot
```

Use `--notify-app codex` if the intended terminal runs Codex. The watcher checks the observed terminal address, exact app identity and idle/done state before sending. Both `--notify-terminal` and `--notify-app` are required together, including when resuming an older command. The watcher continues locally and sends a fixed completion message. The message has no page text or private resource URLs. Keep Herdr and the browser running. Avoid repeatedly reading the screen through the model while this process runs. For non-Herdr hosts, omit notification flags and use the event file with the host's supported callback mechanism; the file alone does not wake an agent.

If `hc` requests approval or reports a blocked target, do not force-send. Reconcile the relay receipt as described in recovery. No internal host API or credential copying is required.
