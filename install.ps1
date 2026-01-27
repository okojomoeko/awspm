$Repo = "okojomoeko/awspm"
$Asset = "awspm-windows-amd64.exe"

# fetch latest version
$LatestRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
$Version = $LatestRelease.tag_name

$Url = "https://github.com/$Repo/releases/download/$Version/$Asset"
$DestDir = "$env:USERPROFILE\.cargo\bin" # Fallback if CARGO_HOME not set, or verify
if (-not (Test-Path $DestDir)) {
    mkdir $DestDir
}
$DestPath = "$DestDir\awspm.exe"

Write-Host "Downloading awspm $Version to $DestPath..."
Invoke-WebRequest -Uri $Url -OutFile $DestPath

Write-Host "awspm installed successfully."
Write-Host "Ensure $DestDir is in your PATH."
