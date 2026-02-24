# scripts/setup.ps1 — Interactive Windows setup for DictateYourTerms
# Downloads a whisper.cpp model and writes the daemon config.

$ErrorActionPreference = "Stop"
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."

# ── Install build dependencies (optional) ────────────────────────────
$answer = Read-Host "`nInstall build dependencies (LLVM, CMake, VS Build Tools)? (y/N)"
if ($answer -eq "y") {
    # LLVM / Clang
    $llvmInstalled = winget list LLVM.LLVM 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Installing LLVM..." -ForegroundColor Green
        winget install LLVM.LLVM
    } else {
        Write-Host "LLVM already installed — skipping." -ForegroundColor Yellow
    }
    [System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "User")
    $env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
    Write-Host "LIBCLANG_PATH set to C:\Program Files\LLVM\bin" -ForegroundColor Green

    # CMake
    $cmakeInstalled = winget list Kitware.CMake 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Installing CMake..." -ForegroundColor Green
        winget install Kitware.CMake
    } else {
        Write-Host "CMake already installed — skipping." -ForegroundColor Yellow
    }

    # Visual Studio Build Tools
    $vsInstalled = winget list Microsoft.VisualStudio.2022.BuildTools 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Installing VS Build Tools..." -ForegroundColor Green
        winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --passive"
    } else {
        Write-Host "VS Build Tools already installed — skipping." -ForegroundColor Yellow
    }

    Write-Host "`nNote: You may need to open a new terminal for PATH changes to take effect." -ForegroundColor Yellow
} else {
    Write-Host "Skipping dependency installation." -ForegroundColor Yellow
}


# ── Model selection menu ─────────────────────────────────────────────
Write-Host "`nSelect a whisper model to download:" -ForegroundColor Cyan
Write-Host "  1) tiny.en   (~75 MB)  - Fastest, lowest accuracy"
Write-Host "  2) base.en   (~142 MB) - Good balance (recommended)"
Write-Host "  3) small.en  (~466 MB) - Better accuracy"
Write-Host "  4) medium.en (~1.5 GB) - High accuracy, slower"
Write-Host ""

$choice = Read-Host "Enter choice [1-4] (default: 2)"
if ($choice -eq "") { $choice = "2" }

# Map choice to model name
switch ($choice) {
    "1" { $ModelName = "tiny.en" }
    "2" { $ModelName = "base.en" }
    "3" { $ModelName = "small.en" }
    "4" { $ModelName = "medium.en" }
    default {
        Write-Host "Invalid choice: $choice. Please enter 1-4." -ForegroundColor Red
        exit 1
    }
}

# ── Download model ───────────────────────────────────────────────────
$modelDir  = "$env:USERPROFILE\.models"
$modelFile = "$modelDir\ggml-$ModelName.bin"
$modelUrl  = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-$ModelName.bin"

New-Item -ItemType Directory -Path $modelDir -Force | Out-Null

$doDownload = $true
if (Test-Path $modelFile) {
    $answer = Read-Host "Model file already exists at $modelFile. Re-download? (y/N)"
    if ($answer -ne "y") { $doDownload = $false }
}

if ($doDownload) {
    Write-Host "Downloading ggml-$ModelName.bin ..." -ForegroundColor Green
    Invoke-WebRequest -Uri $modelUrl -OutFile $modelFile
    Write-Host "Model saved to $modelFile" -ForegroundColor Green
} else {
    Write-Host "Skipping download — using existing model." -ForegroundColor Yellow
}

# ── Config creation ──────────────────────────────────────────────────
$configDir  = "$env:APPDATA\dyt"
$configFile = "$configDir\config.toml"
$templateFile = "$ProjectRoot\config\default.toml"

New-Item -ItemType Directory -Path $configDir -Force | Out-Null

$doConfig = $true
if (Test-Path $configFile) {
    $answer = Read-Host "Config already exists at $configFile. Overwrite? (y/N)"
    if ($answer -ne "y") { $doConfig = $false }
}

if ($doConfig) {
    # Build the TOML-safe path with double backslashes
    $escapedPath = $modelFile.Replace("\", "\\")
    $template = Get-Content $templateFile -Raw
    $config = $template -replace 'model_path = ".*"', "model_path = `"$escapedPath`""
    Set-Content -Path $configFile -Value $config -NoNewline
    Write-Host "Config written to $configFile" -ForegroundColor Green
} else {
    Write-Host "Skipping config — keeping existing file." -ForegroundColor Yellow
}

# ── Build and install binaries (optional) ─────────────────────────────
$binariesInstalled = $false
$answer = Read-Host "`nBuild and install dyt binaries to cargo bin? (y/N)"
if ($answer -eq "y") {
    Write-Host "Installing dyt-daemon..." -ForegroundColor Green
    cargo install --path "$ProjectRoot\dyt-daemon"
    Write-Host "Installing dyt (CLI)..." -ForegroundColor Green
    cargo install --path "$ProjectRoot\dyt-cli"
    $binariesInstalled = $true
    Write-Host "Binaries installed to cargo bin." -ForegroundColor Green
} else {
    Write-Host "Skipping binary installation." -ForegroundColor Yellow
}

# ── Summary ──────────────────────────────────────────────────────────
Write-Host "`nSetup complete!" -ForegroundColor Cyan
Write-Host "  Model:  $modelFile" -ForegroundColor Cyan
Write-Host "  Config: $configFile" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
if ($binariesInstalled) {
    Write-Host "  dyt-daemon                  # start the daemon"
    Write-Host "  dyt --record                # record and transcribe"
} else {
    Write-Host "  cargo build --release"
    Write-Host "  cargo run -p dyt-daemon"
}
Write-Host ""
