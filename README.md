# ThreatBins

A comprehensive Office document SDK and API testing suite built with Rust, designed for defensive security tasks involving safe document creation and conversion.

## Overview

ThreatBins provides a robust SDK for working with various Office document formats, including:
- Microsoft Word (DOCX)
- Microsoft Excel (XLSX)
- Microsoft PowerPoint (PPTX)
- PDF documents
- Email (EML) files
- Markdown conversion

## Features

- **Safe Document Parsing**: Memory-safe operations using Rust's safety guarantees
- **Multiple Format Support**: Comprehensive support for Office formats, PDF, and email
- **CLI Tools**: Command-line interface for document conversion and manipulation
- **API Server**: RESTful API for document operations
- **Markdown Conversion**: Bidirectional conversion between Markdown and Office formats
- **Security Focused**: Designed for defensive security tasks with proper error handling

## Project Structure

```
threatbins/
├── office-sdk/        # Main Rust SDK implementation
│   ├── src/          # Source code
│   ├── examples/     # Usage examples
│   └── tests/        # Integration tests
├── cpp-sdk/          # C++ SDK implementation
├── api-test/         # API testing utilities
└── docs/             # Documentation
```

## Quick Start

### Prerequisites

- Rust 1.70 or higher
- Cargo (Rust package manager)

### Installation

```bash
# Clone the repository
git clone git@github.com:ThreatFlux/threatbins.git
cd threatbins

# Build the project
cd office-sdk
cargo build --release
```

### Basic Usage

#### CLI Tool

```bash
# Convert Markdown to PDF
cargo run --bin office-cli convert document.md --format pdf --title "My Report"

# Convert Markdown to DOCX
cargo run --bin office-cli convert document.md --format docx --title "Business Report"

# Extract text from PDF
cargo run --bin office-cli extract document.pdf

# Get document info
cargo run --bin office-cli info document.pdf
```

#### API Server

```bash
# Start the API server
cargo run --bin office-api-server

# The server will start on http://localhost:3000
```

### Running Examples

```bash
cd office-sdk
cargo run --example docx_example
cargo run --example xlsx_example
cargo run --example pptx_example
cargo run --example pdf_example
cargo run --example markdown_conversion
```

## Development

### Running Tests

```bash
cd office-sdk
cargo test

# Run with output
cargo test -- --nocapture
```

### Building Documentation

```bash
cargo doc --open
```

## API Endpoints

The API server provides the following endpoints:

- `POST /api/convert` - Convert between document formats
- `POST /api/extract` - Extract text from documents
- `GET /api/info` - Get document information
- `POST /api/merge` - Merge multiple documents
- `POST /api/split` - Split documents

See `API_TESTING.md` for detailed API documentation and examples.

## Security

This SDK is designed for defensive security tasks:
- Safe parsing of Office documents
- Secure file handling with proper error management
- Memory-safe operations using Rust's safety guarantees
- Input validation for all document parsing

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues, questions, or suggestions, please open an issue on GitHub.