<#
.SYNOPSIS
    Installs a published snipe-code release binary on Windows.

.DESCRIPTION
    Downloads the zip to a temporary path, verifies the published checksum and the
    cosign bundle when cosign is present, then copies the binary into place. Nothing
    is piped into a shell. A failed check throws without leaving a partial binary
    behind.

    Uninstall: remove the installed snipe-code.exe and its directory.

.EXAMPLE
    irm https://raw.githubusercontent.com/athallaarl66/Snipe-code/main/install.ps1 | iex
#>
[CmdletBinding()]
param(
    [string]$Version = $(if ($env:SNIPE_INSTALL_VERSION) { $env:SNIPE_INSTALL_VERSION } else { 'latest' }),
    [string]$InstallDir = $(if ($env:SNIPE_INSTALL_DIR) { $env:SNIPE_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Programs\snipe-code' }),
    [string]$BaseUrl = $(if ($env:SNIPE_RELEASE_BASE_URL) { $env:SNIPE_RELEASE_BASE_URL } else { 'https://github.com/athallaarl66/Snipe-code/releases/download' })
)

$ErrorActionPreference = 'Stop'
$repo = 'athallaarl66/Snipe-code'

switch ($env:PROCESSOR_ARCHITECTURE) {
    'AMD64' {
        $target = 'x86_64-pc-windows-msvc'
        $binary = 'snipe-code.exe'
    }
    default {
        throw "unsupported architecture: windows/$($env:PROCESSOR_ARCHITECTURE) (no published target)"
    }
}

if ($Version -eq 'latest') {
    $release = Invoke-RestMethod -Headers @{ 'User-Agent' = 'snipe-code-install' } `
        -Uri "https://api.github.com/repos/$repo/releases/latest"
    $Version = $release.tag_name
    if (-not $Version) { throw 'could not resolve the latest release' }
}

$asset = "snipe-code-$Version-$target.zip"
$url = "$BaseUrl/$Version/$asset"
$workspace = Join-Path ([IO.Path]::GetTempPath()) ("snipe-code-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $workspace | Out-Null

try {
    try {
        Invoke-WebRequest -Uri $url -OutFile (Join-Path $workspace $asset) -UseBasicParsing
        Invoke-WebRequest -Uri "$url.sha256" -OutFile (Join-Path $workspace "$asset.sha256") -UseBasicParsing
    }
    catch {
        throw "could not download ${url}: $($_.Exception.Message)"
    }

    $expected = ((Get-Content (Join-Path $workspace "$asset.sha256") -TotalCount 1) -split '\s+')[0].ToLower()
    if (-not $expected) { throw 'the published checksum file is empty' }
    $actual = (Get-FileHash (Join-Path $workspace $asset) -Algorithm SHA256).Hash.ToLower()
    if ($expected -ne $actual) {
        throw "checksum mismatch for $asset (published $expected, downloaded $actual)"
    }

    if (Get-Command cosign -ErrorAction SilentlyContinue) {
        try {
            Invoke-WebRequest -Uri "$url.bundle" -OutFile (Join-Path $workspace "$asset.bundle") -UseBasicParsing
        }
        catch {
            throw "cosign is installed but the signature bundle for $asset could not be downloaded"
        }
        & cosign verify-blob --bundle (Join-Path $workspace "$asset.bundle") `
            --certificate-identity-regexp 'https://github.com/.*/.github/workflows/.*' `
            --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' `
            (Join-Path $workspace $asset) | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "the cosign bundle for $asset did not verify" }
    }

    $extract = Join-Path $workspace 'extract'
    Expand-Archive -Path (Join-Path $workspace $asset) -DestinationPath $extract -Force
    $source = Get-ChildItem -Path $extract -Recurse -Filter $binary | Select-Object -First 1
    if (-not $source) { throw "the archive does not contain $binary" }

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item $source.FullName (Join-Path $InstallDir $binary) -Force

    Write-Host "Installed $binary $Version to $InstallDir"
    $pathEntries = $env:PATH -split ';'
    if ($pathEntries -notcontains $InstallDir) {
        Write-Host "Add it to PATH:  setx PATH `"$InstallDir;`$env:PATH`""
    }
}
finally {
    Remove-Item -Recurse -Force $workspace -ErrorAction SilentlyContinue
}
