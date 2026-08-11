$ErrorActionPreference = "Stop"

$Repository = if ($env:TC_REPOSITORY) { $env:TC_REPOSITORY } else { "mohamed-dev-labs/Thinking-Computer" }
$Version = if ($env:TC_VERSION) { $env:TC_VERSION } else { "latest" }
$InstallDir = if ($env:TC_INSTALL_DIR) { $env:TC_INSTALL_DIR } else { Join-Path $HOME ".thinking-computer\bin" }
$Asset = "thinking-computer-x86_64-pc-windows-msvc.zip"
$BaseUrl = "https://github.com/$Repository/releases"
$Url = if ($Version -eq "latest") { "$BaseUrl/latest/download/$Asset" } else { "$BaseUrl/download/$Version/$Asset" }

$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("thinking-computer-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null
try {
  $Archive = Join-Path $TempDir $Asset
  Invoke-WebRequest -Uri $Url -OutFile $Archive
  Expand-Archive -Path $Archive -DestinationPath $TempDir -Force
  New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
  Copy-Item -Path (Join-Path $TempDir "thinking-computer.exe") -Destination (Join-Path $InstallDir "thinking-computer.exe") -Force
  Write-Output "Installed thinking-computer to $InstallDir\thinking-computer.exe"
  Write-Output "Add $InstallDir to PATH if it is not already available."
}
finally {
  Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
}

