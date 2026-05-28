#!/usr/bin/env pwsh
$Repo = "cirbinius/cirbinius"
$Version = if ($args[0]) { $args[0] } else { "latest" }
$BinDir = if ($env:CIRBINIUS_BIN_DIR) { $env:CIRBINIUS_BIN_DIR } else { "$env:ProgramFiles\CirBinius" }

$Arch = switch ((Get-WmiObject Win32_Processor).Architecture) {
    0 { "x86_64" }
    9 { "x86_64" }
    12 { "arm64" }
    default { throw "Unsupported architecture" }
}

if ($Version -eq "latest") {
    $Url = "https://github.com/$Repo/releases/latest/download/cirbinius-windows-${Arch}.zip"
} else {
    $Url = "https://github.com/$Repo/releases/download/v${Version}/cirbinius-windows-${Arch}.zip"
}

Write-Host "Installing CirBinius v$Version (windows-$Arch)..."

$TmpDir = Join-Path $env:TEMP "cirbinius-install"
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null
$ZipPath = Join-Path $TmpDir "cirbinius.zip"

Invoke-WebRequest -Uri $Url -OutFile $ZipPath
Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
Copy-Item "$TmpDir\cirbinius-api.exe" "$BinDir\" -Force
Copy-Item "$TmpDir\cirbinius.exe" "$BinDir\" -Force -ErrorAction SilentlyContinue

# Add to PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
    $env:Path = "$env:Path;$BinDir"
}

Remove-Item -Recurse -Force $TmpDir

Write-Host "CirBinius installed to $BinDir"
Write-Host "Run 'cirbinius doctor' to verify."
