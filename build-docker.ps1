# PowerShell script to build and run Veridion Nexus Docker container

Write-Host "🐳 Building Veridion Nexus Docker image..." -ForegroundColor Cyan

# Build the Docker image
docker build -t veridion-nexus:latest .

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful!" -ForegroundColor Green
    Write-Host ""
    Write-Host "To run the container:" -ForegroundColor Yellow
    Write-Host "  docker run -p 8080:8080 veridion-nexus:latest" -ForegroundColor White
    Write-Host ""
    Write-Host "Or use docker-compose:" -ForegroundColor Yellow
    Write-Host "  docker-compose up" -ForegroundColor White
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

