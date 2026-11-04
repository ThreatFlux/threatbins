# Shared report document SDK (schema 1)

`office_sdk::report` renders supplied structured facts and wording to **real
PDF, offline HTML and editable widescreen PowerPoint**. The same versioned
input produces all formats. The SDK does not calculate scores, fetch data,
run AI, approve text, authorize tenants, dispatch jobs or store artifacts.

## Minimal call

```rust
use office_sdk::report::*;
# fn example(document: ReportDocument) -> Result<(), Box<dyn std::error::Error>> {
let document_sha256 = report_document_sha256(&document)?;
let mut options = ReportRenderOptions::new(ReportFormat::Pptx);
options.expected_document_sha256 = Some(document_sha256);
let rendered = render_report(&document, &options)?;
let inspection = inspect_report(&rendered.bytes, &rendered.manifest)?;
// Store bytes using the caller's private artifact service; retain the manifest.
assert!(inspection.visual_review_required);
# Ok(())
# }
```

Run a complete example from the repository root:

```sh
cargo run --manifest-path office-sdk/Cargo.toml --example report_render --locked -- \
  office-sdk/tests/fixtures/qbr-report.json output/reports
```

This writes three documents and one JSON manifest per format. The synthetic
QBR fixture is a conformance example, not a dependency on CounterMeasure
business rules. Other report types use the same document blocks.

## Input and compatibility

`ReportDocument` has `schema_version:1`, `document_id`, `revision`, descriptive
`report_type`, `title`, `subtitle`, `period`, `scope`, `comparison`,
`provenance:{evidence_sha256,sources:[{id,revision}]}`, `definitions:[{term,meaning}]`,
`limitations:[string]`, and ordered `sections:[{id,title,blocks}]`.

Each block is one of these tagged objects:

```json
{"type":"paragraph","text":"Approved plain text"}
```

```json
{"type":"table","table":{"caption":"Measured facts","headers":["Scope","Value"],"rows":[["Zone West","64"]]}}
```

```json
{"type":"chart","chart":{"title":"Observed trend","description":"Only recorded points; the missing day is a gap.","labels":["Day 1","Day 2","Day 3"],"series":[{"name":"Readiness","values":[61,null,64]}],"minimum":0,"maximum":100,"target":75,"bands":[{"label":"Critical","start":0},{"label":"At risk","start":40}]}}
```

Charts currently use a line representation. `null` is a missing point, never
zero; gaps break lines. All formats include a complete text data table with
full labels and every supplied value. Target/band definitions are disclosed as
text; HTML also shades bands and each format draws the supplied target.
The renderer preserves the supplied comparison and scope; it never recomputes
or relabels a comparison. An unavailable result should be an explicit plain
text/table value or nullable chart point supplied by the caller.

`report_document_sha256` is SHA-256 of **`serde_json::to_vec(ReportDocument)`**
using this typed schema's field order. It is not an RFC 8785 canonical-JSON
algorithm. Call this function rather than hashing a differently ordered map.
The evidence digest is an opaque caller-provided binding, not proof that the
caller is authorized or that evidence was reviewed. Bind any domain approval
and selected comparison before invoking the renderer.

All input structs reject unknown fields. The new report API is additive;
existing `OfficeDocument`, FilesAPI/REST format enums, converters and builders
retain their existing API. `ReportFormat` is the report-specific format enum;
adding HTML does not silently change legacy REST behavior. Existing
`PptxDocument::set_slide_size` now serializes its actual dimensions, including
when the deck has no slides. Public report structs and fixture JSON define the
wire contract; no environment variable or network connection enables a format.

Core and other contract-only consumers can use `default-features = false,
features = ["report-model"]` for the serde DTOs, `validate_report_document` and
canonical typed digest without PDF, fonts, ZIP, Office, images or service code.
Its normal dependency graph contains only serde/serde_json, SHA-256 and error
support. The `report` feature adds renderers and implies `report-model`.

The default `full` feature preserves the legacy SDK modules, binaries and image
codecs. Service consumers may use `default-features = false, features = ["report"]`
to omit REST/email/CLI/conversion dependencies and unnecessary image codecs.
For example, run the report tests with `cargo test --manifest-path
office-sdk/Cargo.toml --no-default-features --features report --test report_rendering --locked`.

## Template, limits and fonts

`ReportRenderOptions::new(format)` selects the built-in `executive_v1` template.
Other template names fail explicitly. This version uses a 960 × 540 point
landscape print/deck layout, font-measured word wrapping, split long rows,
repeated table headers and complete provenance pages. HTML is responsive with
print styles, semantic headings/tables, SVG descriptions and text chart data.
Text and table cells remain native editable DrawingML in PowerPoint. Charts
contain native `c:lineChart` parts, cached numeric series, internal relationships
and an editable embedded XLSX workbook. No slide screenshots or external links
are used. Definitions, limitations, source versions and exact document/evidence
bindings appear in slides and speaker notes.

Hard limits (callers may reduce them) are 2 MiB serialized input, 32 MiB output,
256 pages/slides, 10,000 table rows and 1,000 chart points in total. Each text
field is at most 20,000 characters; identity fields are at most 256 characters.
There are at most 128 sections, 256 blocks per section, 8 table columns and 8
chart series. Axis and band values must be finite, increasing/in-range where
applicable. Nonrectangular tables, incompatible point counts, XML-illegal
controls and bidi override characters fail before rendering. Oversized titles
or table headers that cannot fit safely also fail rather than clip. PDF charts
include a measured color key for each series; legends that exceed the available
area fail explicitly. Identity footers wrap within three PDF lines or two PPTX
lines and fail if their complete binding cannot fit. The SDK
never truncates a source row or substitutes zero for missing data.

Noto Sans Regular is bundled with SIL OFL 1.1 license and pinned source metadata
under `assets/fonts/`. PDF and HTML embed the font, and layout uses its measured
glyph advances. A licensed custom font may be supplied as `font_data` (at most
10 MiB), with its installed `font_family` name for PowerPoint; missing glyphs fail explicitly. Complex-script shaping is not supported
and is reported as unavailable by `report_capabilities()`. PowerPoint names
the supplied font family but does not embed its binary; installing that font
in PowerPoint/LibreOffice avoids fallback-width differences. Custom fonts do
not grant rights to embed them: the caller must supply an appropriate license.

## Manifest, inspection and Sage integration

`RenderedReport` contains in-memory `bytes` and `ReportManifest`:

- Schema/SDK version, document ID/revision and document/evidence SHA-256.
- Exact template ID, format, MIME type, byte size and output SHA-256.
- Verified PDF/PPTX page/slide count; `pages_or_slides:null` for continuous HTML.
- Structural inspection results, actual native editable content indicators,
  layout-font digest and `visual_review_required:true`.

`render_report` invokes `inspect_report` before returning. Inspection verifies
hash/size and format metadata, parses the PDF/page count, checks the offline
HTML binding, and parses the PPTX XML, all internal relationship targets,
widescreen geometry, slide count and embedded chart workbooks. The inspector
bounds archive parts/expanded bytes and rejects external links/DTD. Its static
HTML profile rejects comments, CDATA and case-insensitive duplicate attributes
to avoid differences between XML tokenization and browser parsing. It is a
structural check; it never claims human approval, numerical correctness of the
source or pixel-perfect visual acceptance. PowerPoint editing/rendering and
visual QA remain acceptance evidence for a consuming report template.

Sage's generic MCP adapter should:

1. Resolve the authorized immutable document from the server-owned pipeline
   binding and map the approved typed content to this schema. Verify the
   source/evidence/template/destination pins using the host's policies.
2. Call `report_capabilities`, verify the required format/template, compute the
   typed document digest and set `expected_document_sha256`. Reduce limits to
   the pipeline's resource budget. Use the same document for all formats.
3. Run bounded blocking SDK work under the host's execution/cancellation policy.
   Do not accept tenant identity, artifact paths or approval claims from prose.
4. Re-run `inspect_report` as desired, persist bytes through Tasks/Forge/private
   artifacts, and return a scoped reference plus the manifest. SDK filenames
   and bytes confer no download authorization.
5. Retain exact selected comparison/report/evidence/approval bindings and
   require actual rendered-artifact review before reporting feature acceptance.

Use one maintained SDK dependency pinned to the reviewed commit/release. Do
not copy these renderers into Core or Sage. MCP transport, tool admission,
authorization, queues, remote cancellation and artifact retention remain host
responsibilities.
