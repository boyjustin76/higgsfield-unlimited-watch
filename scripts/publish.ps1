# Publish the reviewed checkout. Never print tokens or change global Git config.
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path $PSScriptRoot -Parent
Set-Location -LiteralPath $repoRoot
$gh = Get-Command gh -ErrorAction SilentlyContinue
if (-not $gh) {
    $candidate = Join-Path $env:LOCALAPPDATA 'Microsoft\WinGet\Packages\GitHub.cli_Microsoft.Winget.Source_8wekyb3d8bbwe\bin\gh.exe'
    if (Test-Path -LiteralPath $candidate) { $gh = Get-Item -LiteralPath $candidate }
}
if (-not $gh) { throw 'Install GitHub CLI, then open a new terminal.' }
$ghPath = if ($gh.Source) { $gh.Source } else { $gh.FullName }
& $ghPath auth status
if ($LASTEXITCODE -ne 0) { throw 'GitHub authentication required. Run gh auth login --web, then rerun this script.' }
$owner = & $ghPath api user --jq .login
if ($LASTEXITCODE -ne 0 -or $owner.Trim() -ne 'boyjustin76') { throw 'Expected GitHub owner boyjustin76; review ownership before publishing.' }
$dirty = git status --porcelain
if ($dirty) { throw 'Working tree has uncommitted changes. Review and commit before publishing.' }
git rev-parse --verify HEAD | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'No commit available.' }
$remotes = git remote
if ($remotes -contains 'origin') { throw 'origin already exists; inspect the existing remote rather than recreating the repository.' }
& $ghPath repo create boyjustin76/higgsfield-unlimited-watch --public --source . --remote origin --push --description 'Agent skill for Higgsfield web Unlimited: verify the UI, monitor jobs locally, notify your agent.'
if ($LASTEXITCODE -ne 0) { throw 'Publication did not complete. Inspect repository and remote state before retrying.' }
& $ghPath repo edit boyjustin76/higgsfield-unlimited-watch --add-topic higgsfield --add-topic agent-skills --add-topic browser-automation --add-topic python --add-topic watchdog
if ($LASTEXITCODE -ne 0) { Write-Warning 'Repository created; topic update failed.' }
