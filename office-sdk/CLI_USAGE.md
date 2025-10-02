# Office SDK CLI - Usage Guide

The Office SDK CLI is a powerful command-line tool for creating and converting professional Office documents from Markdown, text, and other sources.

## Installation

```bash
cargo build --release
# The binary will be available at target/release/office-cli
```

## Quick Start

### Convert Markdown to PDF
```bash
office-cli convert document.md --format pdf --title "My Report" --author "John Doe"
```

### Convert Markdown to Word Document
```bash
office-cli convert document.md --format docx --title "Business Report"
```

### Convert Markdown to PowerPoint Presentation
```bash
office-cli convert document.md --format pptx --title "Project Overview"
```

### Convert Markdown to Excel Spreadsheet
```bash
office-cli convert document.md --format xlsx --title "Data Analysis"
```

## Commands

### Convert Documents

Convert files between different Office formats:

```bash
# Basic conversion
office-cli convert input.md --format pdf

# With custom metadata
office-cli convert input.md --format docx \
  --title "My Document" \
  --author "Jane Smith" \
  --output custom_name.docx

# Force overwrite existing files
office-cli convert input.md --format pdf --force
```

### Extract Content

Extract text and data from existing documents:

```bash
# Extract text to stdout
office-cli extract document.pdf

# Extract text to file
office-cli extract document.docx --output extracted.txt

# Extract from PowerPoint
office-cli extract presentation.pptx --output slides.txt
```

### Document Information

Get detailed information about documents:

```bash
# Show document stats
office-cli info document.pdf
office-cli info spreadsheet.xlsx
office-cli info presentation.pptx
office-cli info document.docx
```

## Supported Formats

### Input Formats
- **Markdown** (.md, .markdown) - Rich markdown with tables, lists, code blocks
- **Plain Text** (.txt) - Automatically converted to markdown
- **DOCX** - Microsoft Word documents
- **XLSX** - Microsoft Excel spreadsheets
- **PPTX** - Microsoft PowerPoint presentations
- **PDF** - Portable Document Format

### Output Formats
- **PDF** - Advanced PDF with forms, annotations, security
- **DOCX** - Microsoft Word with full formatting
- **PPTX** - Microsoft PowerPoint with slide structure
- **XLSX** - Microsoft Excel with data organization
- **Text** - Plain text extraction

## Features

### Markdown to PDF Conversion
- **Professional formatting** with proper headings and spacing
- **Code blocks** with syntax highlighting support
- **Tables** with proper formatting and borders
- **Lists** (bulleted and numbered) with proper indentation
- **Multi-page support** with automatic page breaks
- **Custom metadata** (title, author, subject, keywords)
- **Security features** (password protection available)

### Markdown to Word (DOCX) Conversion
- **Rich text formatting** with proper styles
- **Heading hierarchy** (H1, H2, H3, etc.)
- **Table of contents** support
- **Professional layout** with consistent spacing
- **Image embedding** (when supported)
- **Metadata integration** with document properties

### Markdown to PowerPoint (PPTX) Conversion
- **Automatic slide creation** from markdown headings
- **Bullet points** converted to PowerPoint lists
- **Title slides** from H1 headings
- **Content slides** from H2+ headings and content
- **Professional presentation layout**
- **Speaker notes** support (when available)

### Markdown to Excel (XLSX) Conversion
- **Table extraction** to spreadsheet format
- **Data organization** with proper row/column structure
- **Multiple worksheets** from different sections
- **Metadata preservation** in document properties

## Advanced Features

### Document Analysis
The CLI provides comprehensive document analysis:

```bash
office-cli info document.pdf
```

Output includes:
- Document type and format
- File size and page count
- Embedded images and multimedia
- Form fields and interactive elements
- Security settings and encryption
- JavaScript and automation features
- Author and creation metadata

### Text Extraction
Extract text content for analysis or processing:

```bash
# Extract all text content
office-cli extract document.pdf

# Save extracted text
office-cli extract document.docx --output content.txt
```

### Batch Processing
Convert multiple files using shell patterns:

```bash
# Convert all markdown files to PDF
for file in *.md; do
    office-cli convert "$file" --format pdf --force
done

# Convert with consistent metadata
for file in *.md; do
    office-cli convert "$file" --format docx \
      --author "Documentation Team" \
      --force
done
```

## Examples

### Example 1: Technical Documentation
```bash
# Create a technical manual from markdown
office-cli convert technical_guide.md \
  --format pdf \
  --title "Technical Documentation" \
  --author "Engineering Team"
```

### Example 2: Business Report
```bash
# Convert business report to Word format
office-cli convert quarterly_report.md \
  --format docx \
  --title "Q4 2024 Business Report" \
  --author "Finance Department"
```

### Example 3: Presentation Slides
```bash
# Create presentation from markdown outline
office-cli convert project_overview.md \
  --format pptx \
  --title "Project Overview Presentation" \
  --author "Project Manager"
```

### Example 4: Data Analysis
```bash
# Convert structured data to spreadsheet
office-cli convert data_analysis.md \
  --format xlsx \
  --title "Market Analysis Q4 2024" \
  --author "Analytics Team"
```

## Tips and Best Practices

### Markdown Structure for Best Results

#### For PDF Documents:
```markdown
# Main Title
## Section Heading
### Subsection

Regular paragraph text with **bold** and *italic* formatting.

- Bullet point 1
- Bullet point 2
- Bullet point 3

| Table Header | Data Column |
|--------------|-------------|
| Row 1        | Value 1     |
| Row 2        | Value 2     |

```code
Code blocks are properly formatted
```
```

#### For Presentations:
```markdown
# Presentation Title
This becomes the title slide

## Slide 1 Title
- Bullet point becomes slide content
- Another bullet point
- Third bullet point

## Slide 2 Title
Content for the second slide

### Subsection
Subsections become bullet points in slides
```

#### For Spreadsheets:
```markdown
# Data Analysis

| Product | Q1 Sales | Q2 Sales | Q3 Sales | Q4 Sales |
|---------|----------|----------|----------|----------|
| Laptop  | 100      | 120      | 110      | 130      |
| Tablet  | 80       | 90       | 85       | 95       |
| Phone   | 200      | 250      | 230      | 280      |

## Summary
Tables in markdown are converted to Excel worksheets
```

### File Organization
- Keep source markdown files organized
- Use descriptive filenames
- Include metadata in frontmatter when possible
- Structure content hierarchically for best conversion results

### Output Optimization
- Use `--force` flag to overwrite existing files
- Specify custom output paths for organization
- Include author and title metadata for professional documents
- Test different formats to find the best fit for your content

## Troubleshooting

### Common Issues

**"Output file exists" Error:**
```bash
# Use --force to overwrite
office-cli convert input.md --format pdf --force
```

**Large File Processing:**
```bash
# Use verbose mode to monitor progress
office-cli convert large_document.md --format pdf --verbose
```

**Unsupported Format:**
```bash
# Check supported formats with --help
office-cli convert --help
```

### File Permissions
Ensure you have read permissions for input files and write permissions for output directories.

### Memory Usage
For very large documents, ensure sufficient system memory is available. The CLI will show warnings for files over 100MB.

## Examples in This Repository

Run these examples to see the CLI in action:

```bash
# Convert the sample markdown to all formats
office-cli convert sample.md --format pdf --title "PDF Demo"
office-cli convert sample.md --format docx --title "Word Demo"
office-cli convert sample.md --format pptx --title "PowerPoint Demo"
office-cli convert sample.md --format xlsx --title "Excel Demo"

# Extract text from created documents
office-cli extract sample.pdf
office-cli extract sample.docx
office-cli extract sample.pptx

# Get document information
office-cli info sample.pdf
office-cli info sample.docx
office-cli info sample.pptx
```

## Integration

The CLI can be integrated into larger workflows:

### GitHub Actions
```yaml
- name: Convert Documentation
  run: |
    office-cli convert README.md --format pdf --title "Project Documentation"
    office-cli convert CHANGELOG.md --format docx --title "Change Log"
```

### Build Scripts
```bash
#!/bin/bash
# Build script example
for doc in docs/*.md; do
    echo "Converting $doc..."
    office-cli convert "$doc" --format pdf --author "Documentation Team" --force
done
echo "Documentation conversion complete!"
```

### Automation Pipelines
The CLI's exit codes and output formatting make it suitable for automated document generation workflows.

## Performance

### Benchmarks
- **Small documents** (< 10KB): < 1 second
- **Medium documents** (10KB - 1MB): 1-5 seconds
- **Large documents** (1MB - 10MB): 5-30 seconds
- **Very large documents** (> 10MB): 30+ seconds

### Optimization
- Use appropriate output formats for content type
- Enable verbose mode for long-running operations
- Consider breaking very large documents into smaller sections

## Support

For issues, feature requests, or questions:
- Check the examples in this repository
- Review the troubleshooting section above
- Ensure all dependencies are properly installed

The Office SDK CLI provides a comprehensive solution for professional document creation and conversion workflows.