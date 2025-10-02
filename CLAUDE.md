# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a test environment and API testing workspace for a Rust-based Office SDK. The main Office SDK implementation is located in the `office-sdk/` subdirectory, while the root contains API testing documentation and utilities. The project focuses on defensive security tasks involving safe document creation and conversion.

## Repository Structure

- **`office-sdk/`** - Main Rust Office SDK implementation with library and CLI
- **`api-test/`** - API testing utilities and scripts
- **`API_TESTING.md`** - Comprehensive API testing documentation and results
- **`CLAUDE.md`** - This guidance file

## Key Commands

### Working with the Office SDK
All Office SDK development happens in the `office-sdk/` directory:

```bash
# Navigate to the SDK directory
cd office-sdk/

# Build the entire project
cargo build

# Build with optimizations
cargo build --release

# Build specific binary targets
cargo build --bin office-cli
cargo build --bin office-api-server

# The CLI binary will be available at target/release/office-cli
# API server binary will be at target/release/office-api-server
```

### Running Tests
```bash
# From office-sdk/ directory
cargo test

# Run tests with output
cargo test -- --nocapture

# Run integration tests specifically
cargo test --test integration_tests
```

### Running Examples
```bash
# From office-sdk/ directory
cargo run --example docx_example
cargo run --example xlsx_example
cargo run --example pptx_example
cargo run --example advanced_docx
cargo run --example advanced_pdf
cargo run --example advanced_pptx_simple
cargo run --example advanced_xlsx_simple
cargo run --example markdown_conversion
cargo run --example pdf_example
cargo run --example eml_example
cargo run --example files_api_example
```

### Using the CLI
```bash
# From office-sdk/ directory - The main CLI is office-cli-simple.rs
cargo run --bin office-cli convert document.md --format pdf --title "My Report" --author "John Doe"
cargo run --bin office-cli convert document.md --format docx --title "Business Report"
cargo run --bin office-cli extract document.pdf
cargo run --bin office-cli info document.pdf

# Alternative CLI (more features)
cargo run --bin office-cli-simple convert document.md --format pdf
```

### API Server Commands
The SDK includes multiple API server implementations:

```bash
# From office-sdk/ directory
cargo run --bin office-api-server     # Main API server
cargo run --bin simple-api-server     # Simplified API
cargo run --bin standalone-api        # Standalone version
cargo run --bin minimal-api           # Minimal implementation
```

## Architecture

### Core Structure
The Office SDK (in `office-sdk/` directory) follows a modular architecture with separate modules for each document format:

- **`office-sdk/src/lib.rs`** - Main library entry point defining the `OfficeDocument` trait
- **`office-sdk/src/docx.rs`** - Microsoft Word document handling
- **`office-sdk/src/xlsx.rs`** - Microsoft Excel spreadsheet handling
- **`office-sdk/src/pptx.rs`** - Microsoft PowerPoint presentation handling
- **`office-sdk/src/pdf.rs`** - PDF document handling
- **`office-sdk/src/eml.rs`** - Email (EML) document handling
- **`office-sdk/src/markdown.rs`** - Markdown parsing and conversion
- **`office-sdk/src/files.rs`** - File handling and storage utilities
- **`office-sdk/src/error.rs`** - Error types and handling
- **`office-sdk/src/utils.rs`** - Shared utilities

### Key Traits
All document types implement the `OfficeDocument` trait providing consistent APIs:
```rust
pub trait OfficeDocument {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> where Self: Sized;
    fn from_bytes(data: &[u8]) -> Result<Self> where Self: Sized;
    fn to_bytes(&self) -> Result<Vec<u8>>;
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}
```

### Document Format Support
- **DOCX**: Full Word document creation with images, tables, VBA macros, metadata, page layout
- **XLSX**: Excel spreadsheet creation with multiple worksheets
- **PPTX**: PowerPoint presentation creation with slide management
- **PDF**: Advanced PDF creation with forms, annotations, security features
- **EML**: Email document parsing and creation
- **Markdown**: Bidirectional conversion with DOCX format

### Binary Applications
The SDK includes multiple binary applications in `office-sdk/src/bin/`:

- **`office-cli-simple.rs`** - Main CLI tool (mapped to `office-cli` binary) with `convert`, `extract`, and `info` commands
- **`office-cli.rs`** - Alternative CLI with extended features
- **`api-server.rs`** - Full-featured API server for document operations (mapped to `office-api-server`)
- **`simple-api-server.rs`** - Simplified API server implementation
- **`standalone-api.rs`** - Standalone API server
- **`minimal-api.rs`** - Minimal API server implementation

### API Testing Environment
The root directory contains comprehensive API testing resources:

- **`API_TESTING.md`** - Detailed testing results and curl examples
- **`api-test/`** - Testing utilities and scripts
- All API endpoints tested and verified with real document generation

## Development Guidelines

### File Organization
- Core document format implementations are in individual `.rs` files in `office-sdk/src/`
- Binary applications are in `office-sdk/src/bin/`
- Examples demonstrating each format are in `office-sdk/examples/`
- Integration tests are in `office-sdk/tests/integration_tests.rs`
- API testing documentation and utilities are in the root directory

### Dependencies
The project uses stable, well-maintained crates (see `office-sdk/Cargo.toml`):
- **Document processing**: `zip`, `quick-xml`, `pulldown-cmark`, `serde-xml-rs`
- **Error management**: `thiserror`, `anyhow`
- **Serialization**: `serde`, `serde_json`
- **CLI interface**: `clap` with derive features (colors, suggestions)
- **PDF generation**: `printpdf`, `lopdf`, `pdf-writer`
- **API server**: `tokio`, `warp` for async HTTP server
- **Email handling**: `lettre`, `mail-parser`, `mime`
- **Image processing**: `image` for format support and auto-sizing
- **CLI utilities**: `indicatif`, `console`, `dialoguer`, `colored`, `dirs`, `walkdir`
- **Utilities**: `uuid`, `chrono`, `base64`, `regex`, `md5`, `time`, `bytes`, `futures-util`

### Security Focus
This SDK is designed for defensive security tasks:
- Safe parsing of Office documents
- Secure file handling with proper error management
- Memory-safe operations using Rust's safety guarantees
- Input validation for all document parsing

### Testing
- Comprehensive unit tests within each module in `office-sdk/`
- Integration tests in `office-sdk/tests/integration_tests.rs`
- Example programs in `office-sdk/examples/` that serve as functional tests
- API testing documented in `API_TESTING.md` with curl examples
- Real document generation verified for all formats
- 90%+ code coverage target

## Working with Document Formats

### Adding New Features
When extending document format support in `office-sdk/`:
1. Implement the `OfficeDocument` trait for consistency
2. Add comprehensive error handling using `OfficeError`
3. Include unit tests for all new functionality
4. Create example code demonstrating usage in `examples/`
5. Update integration tests as needed
6. Test API endpoints if adding API support

### Common Patterns
- All document types use ZIP-based compressed formats (except PDF/EML)
- XML parsing is handled consistently with `quick-xml`
- Metadata is managed through document properties
- Image embedding uses base64 encoding within XML structures

## API Testing

The repository includes comprehensive API testing:
- All endpoints tested with real document generation
- Curl examples provided in `API_TESTING.md`
- Multiple API server implementations available
- Test results show 100% endpoint coverage with binary document verification

## Working Directory

When working with this codebase:
- Most development happens in the `office-sdk/` subdirectory
- API testing documentation is in the root directory
- Use `cd office-sdk/` before running cargo commands
- Check `API_TESTING.md` for API usage examples and test results