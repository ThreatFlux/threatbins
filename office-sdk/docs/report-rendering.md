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
  office-sdk/tests/fixtures/executive-report.json output/reports
```

This writes three documents and one JSON manifest per format. The synthetic
executive report fixture demonstrates the generic document contract. Applications
supply their own report types, facts and business rules using the same blocks.

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

Internal references use caller-owned section or table-row IDs:

```json
{"type":"links","links":[{"label":"Review record 42","target_id":"record:42"}]}
```

```json
{"type":"anchored_table","table":{"caption":"Records","headers":["Record","Action"],"rows":[["42","Review the retained evidence"]]},"row_ids":["record:42"]}
```

Row IDs must match the row count and be unique across all section and row IDs.
Missing targets and empty content targets are rejected before rendering. Links
remain internal: HTML uses fragment anchors, PDF uses native GoTo annotations,
and PowerPoint uses native slide-jump relationships to the page containing that
exact row. Oversized and split rows retain their original first destination.
Links do not fetch URLs or add external package relationships. Legacy blocks
retain their serialization and digest. SDK 0.2 adds these tagged variants; Rust
consumers that match `ReportBlock` exhaustively must handle both new variants.

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

Contract-only consumers can use `default-features = false,
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

`ReportRenderOptions::new(format)` defaults to `executive_v1`. This template opens
with a report overview and paginates the individual blocks. To select
`section_first_v1`, set `options.template = SECTION_FIRST_TEMPLATE.into()`.
Both templates accept the same schema-1 document and produce the same typed
document digest. Unknown template IDs fail explicitly.

With `section_first_v1`, supply at least one section and place your opening
narrative first. PDF starts with that section, then presents report context and
the remaining sections. The opening page reserves its footer for the full
period and comparison. Document and revision IDs remain in the provenance
pages and embedded manifest binding. Long tables repeat their headers on later
pages, and internal links reach the exact target page and row. HTML starts with the same
section, followed by context, in a single responsive document with printable
styles and internal anchors. Both carry complete chart data, missing observations,
definitions, limitations and provenance.

PowerPoint puts each section on exactly one editable 16:9 slide, in input
order. Period and comparison appear on each slide. The layout measures text
and cells, and can arrange consecutive charts in a compact grid. Each chart
is a native line chart linked to an editable workbook with the full
observation labels and numeric data. Blank observations remain gaps in the
chart and workbook. Long or numerous category labels stay available in chart
data and speaker notes when the visible axis has insufficient space. Long
series names can appear in an editable color key on the slide. Speaker
notes also carry the full report context, definitions, limitations, source
revisions and document/evidence hashes. When a full native table exceeds the
slide, a native preview indicates how many rows are visible and the notes
retain every header, row, cell and row anchor. An internal row link jumps to
the slide with that table or its notes. When compact layouts and previews
still leave content that cannot fit, rendering returns the section ID and
asks the caller to split the content into additional sections.

Hard limits (callers may reduce them) are 2 MiB serialized input, 32 MiB output,
256 pages/slides, 10,000 table rows and 1,000 chart points in total. Continuous
section-first HTML has no page count and uses the byte and content limits. Each text
field is at most 20,000 characters; identity fields are at most 256 characters.
There are at most 128 sections, 256 blocks per section, 8 table columns and 8
chart series. Axis and band values must be finite, increasing/in-range where
applicable. Nonrectangular tables, incompatible point counts, XML-illegal
controls and bidi override characters fail before rendering. Oversized titles
or table headers that cannot fit safely also fail rather than clip. PDF charts
include a measured color key for each series; legends that exceed the available
area fail explicitly. The executive template limits identity footers to
three PDF lines or two PPTX lines. The section-first PDF opening footer
fits up to four measured lines of context and fails explicitly when that
context cannot fit. The SDK
never truncates a source row or substitutes zero for missing data.

Noto Sans Regular is bundled with SIL OFL 1.1 license and pinned source metadata
under `assets/fonts/`. PDF and HTML embed the font, and layout uses its measured
glyph advances. A licensed custom font may be supplied as `font_data` (at most
10 MiB), with its installed `font_family` name for PowerPoint; missing glyphs fail explicitly. Complex-script shaping is not supported
and is reported as unavailable by `report_capabilities()`. PowerPoint names
the supplied font family but does not embed its binary; installing that font
in PowerPoint/LibreOffice avoids fallback-width differences. Custom fonts do
not grant rights to embed them: the caller must supply an appropriate license.

## Manifest, inspection and host integration

`RenderedReport` contains in-memory `bytes` and `ReportManifest`:

- Schema/SDK version, document ID/revision and document/evidence SHA-256.
- Exact template ID, format, MIME type, byte size and output SHA-256.
- Verified PDF/PPTX page/slide count; `pages_or_slides:null` for continuous HTML.
- Structural inspection results, actual native editable content indicators,
  layout-font digest and `visual_review_required:true`.

`render_report` invokes `inspect_report` before returning. Inspection verifies
hash/size and format metadata, parses the PDF/page count, checks the offline
HTML binding, and parses the PPTX XML, all internal relationship targets,
widescreen geometry, slide count and embedded chart workbooks. For
`section_first_v1`, all three formats also carry an embedded binding for the
manifest's template, SDK, source identity, hashes, format, count and layout
font. The inspector verifies the section-first HTML reading order and the
ordered PPTX slide, chart, workbook, notes and navigation references. Existing
`executive_v1` artifacts continue to use their historical inspection profile.
An embedded binding is an integrity check against the supplied manifest.
The inspector
bounds archive parts/expanded bytes and rejects external links/DTD. Its static
HTML profile rejects comments, CDATA and case-insensitive duplicate attributes
to avoid differences between XML tokenization and browser parsing. It is a
structural check; it never claims human approval, numerical correctness of the
source or pixel-perfect visual acceptance. PowerPoint editing/rendering and
visual QA remain acceptance evidence for a consuming report template.

The report inspector also rejects incomplete default DrawingML theme style
lists. Each fill, line, effect and background-fill list supplies at least three
entries. CI independently validates the generated presentation and its embedded
workbooks with Microsoft's Open XML schemas; this does not require .NET in SDK
consumers. With .NET SDK 11.0.100 installed (matching the non-rolling SDK pin in
`tools/pptx-validation/global.json`), run `make validate-pptx` from the
repository root to repeat that check. Validator packages and the SDK are pinned
under `tools/pptx-validation/`.

A host application or MCP adapter should:

1. Resolve the authorized immutable document from the server-owned pipeline
   binding and map the approved typed content to this schema. Verify the
   source/evidence/template/destination pins using the host's policies.
2. Call `report_capabilities`, verify the required format/template, compute the
   typed document digest and set `expected_document_sha256`. Reduce limits to
   the pipeline's resource budget. Use the same document for all formats.
3. Run bounded blocking SDK work under the host's execution/cancellation policy.
   Do not accept tenant identity, artifact paths or approval claims from prose.
4. Re-run `inspect_report` as desired, persist bytes through the host's private
   artifact service, and return a scoped reference plus the manifest. SDK
   filenames and bytes confer no download authorization.
5. Retain exact selected comparison/report/evidence/approval bindings and
   require actual rendered-artifact review before reporting feature acceptance.

Use one maintained SDK dependency pinned to the reviewed commit/release. Do
not copy these renderers into consuming applications. MCP transport, tool admission,
authorization, queues, remote cancellation and artifact retention remain host
responsibilities.
