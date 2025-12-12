# Veridion Nexus Platform Stop Script
# This script stops all components of the Veridion Nexus platform

Write-Host "🛑 Stopping Veridion Nexus Platform..." -ForegroundColor Yellow
Write-Host ""

# Navigate to project root
$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $projectRoot

# Stop Docker containers
Write-Host "📦 Stopping Docker containers..." -ForegroundColor Yellow
docker-compose down

Write-Host "`n✅ Platform stopped!" -ForegroundColor Green
Write-Host "`n💡 Note: Backend and Frontend windows need to be closed manually (Ctrl+C in each window)" -ForegroundColor Gray
Write-Host ""

