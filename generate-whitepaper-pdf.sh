#!/bin/bash
# Bash script to convert Veridion Whitepaper Markdown to PDF
# Requires: pandoc (https://pandoc.org/installing.html)

MARKDOWN_FILE="VERIDION_NEXUS_WHITEPAPER.md"
PDF_FILE="VERIDION_NEXUS_WHITEPAPER.pdf"

echo "📄 Generating PDF from Markdown..."

# Check if pandoc is installed
if ! command -v pandoc &> /dev/null; then
    echo "❌ Pandoc is not installed!"
    echo ""
    echo "Please install pandoc:"
    echo "  macOS: brew install pandoc"
    echo "  Ubuntu/Debian: sudo apt-get install pandoc"
    echo "  Or download from: https://pandoc.org/installing.html"
    exit 1
fi

# Convert markdown to PDF using pandoc
pandoc "$MARKDOWN_FILE" \
    -o "$PDF_FILE" \
    --pdf-engine=wkhtmltopdf \
    -V geometry:margin=1in \
    -V fontsize=11pt \
    -V documentclass=article \
    --toc \
    --toc-depth=2 \
    -V colorlinks=true \
    --highlight-style=tango

if [ $? -eq 0 ]; then
    echo "✅ PDF generated successfully: $PDF_FILE"
else
    echo "❌ PDF generation failed!"
    echo ""
    echo "Alternative: Use an online Markdown to PDF converter:"
    echo "  - https://www.markdowntopdf.com/"
    echo "  - https://dillinger.io/ (Export as PDF)"
    exit 1
fi

