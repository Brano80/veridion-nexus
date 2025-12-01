# PowerShell script to convert Veridion Whitepaper Markdown to HTML
# This works without any PDF engine - you can then print to PDF from browser

$markdownFile = "VERIDION_NEXUS_WHITEPAPER.md"
$htmlFile = "VERIDION_NEXUS_WHITEPAPER.html"

Write-Host "📄 Generating HTML from Markdown..." -ForegroundColor Cyan

# Refresh PATH environment variable
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

# Check if pandoc is installed
$pandocInstalled = Get-Command pandoc -ErrorAction SilentlyContinue

if (-not $pandocInstalled) {
    Write-Host "❌ Pandoc is not found!" -ForegroundColor Red
    Write-Host "Please restart your terminal and try again, or install Pandoc." -ForegroundColor Yellow
    exit 1
}

# Convert markdown to HTML (no PDF engine needed)
pandoc $markdownFile `
    -o $htmlFile `
    --standalone `
    --toc `
    --toc-depth=2 `
    --css=https://cdn.jsdelivr.net/npm/github-markdown-css@5/github-markdown.min.css `
    --metadata title="Veridion Nexus Investment Whitepaper" `
    -V lang=en `
    --highlight-style=tango

if ($LASTEXITCODE -eq 0 -and (Test-Path $htmlFile)) {
    Write-Host "✅ HTML generated successfully: $htmlFile" -ForegroundColor Green
    Write-Host ""
    Write-Host "To convert to PDF:" -ForegroundColor Yellow
    Write-Host "1. Open $htmlFile in your browser" -ForegroundColor White
    Write-Host "2. Press Ctrl+P (Print)" -ForegroundColor White
    Write-Host "3. Select 'Save as PDF' as the destination" -ForegroundColor White
    Write-Host "4. Click Save" -ForegroundColor White
    Write-Host ""
    Write-Host "Or use an online converter:" -ForegroundColor Yellow
    Write-Host "  - https://www.markdowntopdf.com/" -ForegroundColor Cyan
    Write-Host "  - https://dillinger.io/" -ForegroundColor Cyan
} else {
    Write-Host "❌ HTML generation failed!" -ForegroundColor Red
    exit 1
}

