# Install or update the Seseragi CLI and LSP on Windows.
#
#   powershell -ExecutionPolicy Bypass -c "irm https://github.com/KentaroMorishita/seseragi/releases/latest/download/install.ps1 | iex"
#
# Environment variables:
#   SESERAGI_VERSION       version to install, such as 0.61.165 (default: latest)
#   SESERAGI_INSTALL       install root; binaries go to $env:SESERAGI_INSTALL\bin
#                          (default: $HOME\.seseragi)
#   SESERAGI_DOWNLOAD_URL  base URL, or local directory, that holds the release
#                          archives (default: the GitHub release for the version)
#   SESERAGI_NO_PATH       set to 1 to leave the user PATH unchanged

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$repository = 'https://github.com/KentaroMorishita/seseragi'

# Windows PowerShell 5.1 may not offer TLS 1.2 by default.
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

# Throw instead of exit: under `irm ... | iex`, exit would close the caller's
# PowerShell window.
function Fail([string]$message) {
  throw "seseragi install: $message"
}

function Get-ReleaseFile([string]$name, [string]$destination) {
  if ($baseUrl -match '^https?://') {
    Invoke-WebRequest -Uri "$baseUrl/$name" -OutFile $destination -UseBasicParsing
  } else {
    Copy-Item -Path (Join-Path $baseUrl $name) -Destination $destination
  }
}

$cpu = $env:PROCESSOR_ARCHITECTURE
if ($cpu -ne 'AMD64') {
  Fail "no prebuilt CLI for Windows $cpu yet. Build from source: cargo install --git $repository --locked seseragi-cli"
}
$target = 'win32-x64'

$version = $env:SESERAGI_VERSION
if (-not $version) {
  try {
    $response = Invoke-WebRequest -Uri "$repository/releases/latest" -UseBasicParsing -Method Head
    $latestUrl = $response.BaseResponse.ResponseUri
    if (-not $latestUrl) { $latestUrl = $response.BaseResponse.RequestMessage.RequestUri }
    $version = ($latestUrl.AbsoluteUri -split '/')[-1]
  } catch {
    Fail "could not find the latest release: $_"
  }
}
$version = $version.TrimStart('v')
if ($version -notmatch '^[0-9]+(\.[0-9]+)*$') { Fail "invalid version: $version" }

$archive = "seseragi-v$version-$target.zip"
$baseUrl = $env:SESERAGI_DOWNLOAD_URL
if (-not $baseUrl) { $baseUrl = "$repository/releases/download/v$version" }
$installRoot = $env:SESERAGI_INSTALL
if (-not $installRoot) { $installRoot = Join-Path $HOME '.seseragi' }
$binDir = Join-Path $installRoot 'bin'

$work = Join-Path ([System.IO.Path]::GetTempPath()) ("seseragi-install-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Path $work | Out-Null
try {
  Write-Host "Downloading Seseragi $version for $target"
  $archivePath = Join-Path $work $archive
  $checksumPath = "$archivePath.sha256"
  try {
    Get-ReleaseFile $archive $archivePath
  } catch {
    Fail "could not download $baseUrl/$archive"
  }
  try {
    Get-ReleaseFile "$archive.sha256" $checksumPath
  } catch {
    Fail "could not download the checksum $baseUrl/$archive.sha256"
  }

  $expected = ((Get-Content -Raw $checksumPath).Trim() -split '\s+')[0].ToLowerInvariant()
  if ($expected -notmatch '^[0-9a-f]{64}$') { Fail 'the checksum file is malformed' }
  $actual = (Get-FileHash -Algorithm SHA256 -Path $archivePath).Hash.ToLowerInvariant()
  if ($expected -ne $actual) {
    Fail "checksum mismatch for $archive (expected $expected, got $actual)"
  }

  $extract = Join-Path $work 'extract'
  Expand-Archive -Path $archivePath -DestinationPath $extract
  foreach ($binary in 'seseragi.exe', 'seseragi-lsp.exe') {
    if (-not (Test-Path (Join-Path $extract $binary))) { Fail "$archive does not contain $binary" }
  }

  New-Item -ItemType Directory -Force -Path $binDir | Out-Null
  # Files set aside by earlier updates; ones still in use stay until next time.
  Get-ChildItem -Path $binDir -Filter '*.old' -ErrorAction SilentlyContinue |
    Remove-Item -Force -ErrorAction SilentlyContinue
  foreach ($binary in 'seseragi.exe', 'seseragi-lsp.exe') {
    $destination = Join-Path $binDir $binary
    # A running .exe cannot be overwritten but can be renamed. Use a fresh
    # name so a still-running copy from an earlier update does not block it.
    if (Test-Path $destination) {
      Move-Item -Force $destination "$destination.$([System.Guid]::NewGuid().ToString('N')).old"
    }
    Copy-Item (Join-Path $extract $binary) $destination
  }
  Get-ChildItem -Path $binDir -Filter '*.old' -ErrorAction SilentlyContinue |
    Remove-Item -Force -ErrorAction SilentlyContinue

  # The archive's license notice covers Unicode data inside the binaries.
  $license = Join-Path $extract 'UNICODE-LICENSE'
  if (Test-Path $license) {
    Copy-Item -Force $license (Join-Path $installRoot 'UNICODE-LICENSE')
  }
} finally {
  Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $work
}

$installed = & (Join-Path $binDir 'seseragi.exe') --version
if ($LASTEXITCODE -ne 0) { Fail 'the installed CLI does not run' }
Write-Host "Installed $installed to $binDir"

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$entries = @($userPath -split ';' | Where-Object { $_ })
if ($entries -notcontains $binDir) {
  if ($env:SESERAGI_NO_PATH -eq '1') {
    Write-Host ''
    Write-Host "Add $binDir to your PATH to run seseragi."
  } else {
    [Environment]::SetEnvironmentVariable('Path', (($entries + $binDir) -join ';'), 'User')
    Write-Host ''
    Write-Host "Added $binDir to your user PATH. Open a new terminal to use seseragi."
  }
}

if (-not (Get-Command bun -ErrorAction SilentlyContinue)) {
  Write-Host ''
  Write-Host 'Running programs needs Bun 1.4.4 or later: https://bun.sh/docs/installation'
}
