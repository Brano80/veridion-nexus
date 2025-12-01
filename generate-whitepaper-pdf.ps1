# PowerShell script to convert Veridion Whitepaper Markdown to PDF
# Requires: pandoc (https://pandoc.org/installing.html)

$markdownFile = "VERIDION_NEXUS_WHITEPAPER.md"
$pdfFile = "VERIDION_NEXUS_WHITEPAPER.pdf"

Write-Host "📄 Generating PDF from Markdown..." -ForegroundColor Cyan

# Refresh PATH environment variable to pick up newly installed programs
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

# Check if pandoc is installed
$pandocInstalled = Get-Command pandoc -ErrorAction SilentlyContinue

# If not found, try common installation paths
if (-not $pandocInstalled) {
    $commonPaths = @(
        "$env:LOCALAPPDATA\Microsoft\WindowsApps\pandoc.exe",
        "$env:ProgramFiles\Pandoc\pandoc.exe",
        "$env:ProgramFiles(x86)\Pandoc\pandoc.exe",
        "$env:USERPROFILE\AppData\Local\Microsoft\WindowsApps\pandoc.exe"
    )
    
    foreach ($path in $commonPaths) {
        if (Test-Path $path) {
            Write-Host "✅ Found Pandoc at: $path" -ForegroundColor Green
            $pandocPath = $path
            break
        }
    }
    
    if (-not $pandocPath) {
        Write-Host "❌ Pandoc is not found in PATH!" -ForegroundColor Red
        Write-Host ""
        Write-Host "Pandoc may be installed but not in your PATH." -ForegroundColor Yellow
        Write-Host "Try one of these solutions:" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "1. Restart your terminal/PowerShell and try again" -ForegroundColor White
        Write-Host "2. Manually add Pandoc to PATH:" -ForegroundColor White
        Write-Host "   - Find Pandoc installation (usually in AppData\Local\Microsoft\WindowsApps)" -ForegroundColor Gray
        Write-Host "   - Add it to System Environment Variables" -ForegroundColor Gray
        Write-Host ""
        Write-Host "3. Use an online converter instead:" -ForegroundColor White
        Write-Host "   - https://www.markdowntopdf.com/" -ForegroundColor Cyan
        Write-Host "   - https://dillinger.io/ (Export as PDF)" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "4. Use VS Code Markdown PDF extension" -ForegroundColor White
        exit 1
    }
} else {
    $pandocPath = "pandoc"
}

# Convert markdown to PDF using pandoc
# Try different PDF engines in order of preference
$pdfEngines = @("wkhtmltopdf", "xelatex", "pdflatex", "weasyprint")

$conversionSuccess = $false
foreach ($engine in $pdfEngines) {
    Write-Host "Attempting conversion with $engine..." -ForegroundColor Yellow
    
    if ($pandocPath -eq "pandoc") {
        $result = & pandoc $markdownFile -o $pdfFile --pdf-engine=$engine -V geometry:margin=1in -V fontsize=11pt -V documentclass=article --toc --toc-depth=2 -V colorlinks=true --highlight-style=tango 2>&1
    } else {
        $result = & $pandocPath $markdownFile -o $pdfFile --pdf-engine=$engine -V geometry:margin=1in -V fontsize=11pt -V documentclass=article --toc --toc-depth=2 -V colorlinks=true --highlight-style=tango 2>&1
    }
    
    if ($LASTEXITCODE -eq 0 -and (Test-Path $pdfFile)) {
        $conversionSuccess = $true
        break
    } else {
        Write-Host "  $engine failed, trying next engine..." -ForegroundColor Gray
    }
}

if (-not $conversionSuccess) {
    # Fallback: Try without specifying PDF engine (let pandoc choose)
    Write-Host "Trying conversion without specifying PDF engine..." -ForegroundColor Yellow
    if ($pandocPath -eq "pandoc") {
        & pandoc $markdownFile -o $pdfFile -V geometry:margin=1in -V fontsize=11pt --toc --toc-depth=2 2>&1 | Out-Null
    } else {
        & $pandocPath $markdownFile -o $pdfFile -V geometry:margin=1in -V fontsize=11pt --toc --toc-depth=2 2>&1 | Out-Null
    }
    
    if ($LASTEXITCODE -eq 0 -and (Test-Path $pdfFile)) {
        $conversionSuccess = $true
    }
}

if ($conversionSuccess) {
    Write-Host "✅ PDF generated successfully: $pdfFile" -ForegroundColor Green
    Write-Host "📄 File location: $(Resolve-Path $pdfFile)" -ForegroundColor Cyan
} else {
    Write-Host "❌ PDF generation failed!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Pandoc may be missing a PDF engine. Try installing one:" -ForegroundColor Yellow
    Write-Host "  - wkhtmltopdf: winget install wkhtmltopdf" -ForegroundColor White
    Write-Host "  - Or use MiKTeX/TeX Live for LaTeX engines" -ForegroundColor White
    Write-Host ""
    Write-Host "Alternative: Use an online Markdown to PDF converter:" -ForegroundColor Yellow
    Write-Host "  - https://www.markdowntopdf.com/" -ForegroundColor Cyan
    Write-Host "  - https://dillinger.io/ (Export as PDF)" -ForegroundColor Cyan
    Write-Host "  - VS Code: Install 'Markdown PDF' extension" -ForegroundColor Cyan
    exit 1
}

