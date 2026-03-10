# windows-search-tool Performance Benchmark Script

$exePath = "src-tauri/target/release/windows-search-tool.exe"

if (-not (Test-Path $exePath)) {
    Write-Host "Error: Executable not found at $exePath. Please run 'npm run tauri build' first." -ForegroundColor Red
    exit 1
}

Write-Host "--- windows-search-tool Benchmark ---" -ForegroundColor Cyan

# 1. Binary Size
$size = (Get-Item $exePath).Length / 1MB
$sizeFormatted = "{0:N2} MB" -f $size
Write-Host "Binary Size: $sizeFormatted"

# 2. Startup & Idle Memory
Write-Host "Launching app for memory and startup measurement..."
$startTime = Get-Date
$process = Start-Process $exePath -PassThru
Start-Sleep -Seconds 5 # Give it time to build index

if ($process) {
    $mem = (Get-Process -Id $process.Id).WorkingSet64 / 1MB
    $memFormatted = "{0:N2} MB" -f $mem
    Write-Host "Idle Memory Usage: $memFormatted"
    
    # 3. Shutdown
    Stop-Process -Id $process.Id -Force
    Write-Host "Benchmark complete."
} else {
    Write-Host "Failed to start process." -ForegroundColor Red
}

Write-Host "`nSummary for README:" -ForegroundColor Green
Write-Host "| Binary size | $sizeFormatted |"
Write-Host "| Memory usage (idle) | $memFormatted |"
