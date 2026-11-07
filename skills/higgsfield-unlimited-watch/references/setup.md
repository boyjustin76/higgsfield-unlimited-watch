# Setup and commands

The skill folder is self-contained. Paths below assume a repository checkout; after installing only the skill, replace `skills/higgsfield-unlimited-watch` with its actual location.

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

Sign in directly. Unsupported terminal means missing Kitty graphics support; Noctty worked. Restart the browser after font installation if Korean text remains broken. Windows Chrome and the WSL browser do not automatically share authentication. See [terminal-browser](https://github.com/zenbu-labs/terminal-browser) for current installer support.

## Herdr background monitor

On herdr-core desktop, use `hc`, not the older `herdr` multiplexer. Typical Windows location:
`$env:LOCALAPPDATA\Programs\herdr-core\hc.exe`.

```powershell
$hc = Join-Path $env:LOCALAPPDATA 'Programs\herdr-core\hc.exe'
& $hc ls
```

Select your actual receiving agent terminal ID. Do not copy another machine's ID. Start from the checkout directory, substituting UUID and terminal ID:

```powershell
& $hc run --bg --no-notify --name higgs-watch python skills/higgsfield-unlimited-watch/scripts/huw.py watch --config config.local.json --asset-id RESULT_UUID --state-dir local/job-001 --hc $hc --notify-terminal AGENT_TERMINAL_ID
```

The watcher continues locally and sends a fixed completion message. The message has no page text or private resource URLs. Keep Herdr and the browser running. Avoid repeatedly reading the screen through the model while this process runs. For non-Herdr hosts, omit notification flags and use the event file with the host's supported callback mechanism; the file alone does not wake an agent.

If `hc` requests approval or reports a blocked target, do not force-send. Reconcile the relay receipt as described in recovery. No internal host API or credential copying is required.
