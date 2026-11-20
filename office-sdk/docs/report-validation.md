# Report SDK conformance evidence

Synthetic fixture: [`tests/fixtures/executive-report.json`](../tests/fixtures/executive-report.json).
The example produces PDF, HTML, PPTX and per-format manifests from the same
schema-1 document. No customer data or live report approval is involved.

Validated locally before the SDK implementation commit:

- Repository-owned `make ci` passed on Rust 1.99.1 and MSRV 1.95.0: formatting,
  all-target/all-feature checks, **152 passing tests**, documentation and both
  lockfile audits. `paste` has the inherited allowed unmaintained warning;
  there are no audit vulnerabilities. Patched the inherited
  `pulldown-cmark` 0.13.5 advisory by updating to 0.13.6.
- Contract-only `report-model` tests passed (2), and its normal dependency
  tree excludes PDF/Office/ZIP/font/image/service code.
- Report-only rendering tests passed (19), including shared document digests,
  PDF text/Unicode parity, native editable chart/table/workbook parts, missing
  point semantics, HTML escaping, source/output limits, changed-byte rejection,
  long-row pagination, and new/imported slide-dimension preservation.
- Rust 1.97.1 consumer checks passed independently for both `report` and
  `report-model` with default features disabled. The report-only normal tree
  excludes REST/email/CLI/Markdown and AVIF dependencies.
- Independent review added failing-before/passing-after checks for nested workbook expansion and mixed-case active HTML, plus external workbook links, event attributes and fake policy declarations. Inspection now budgets nested packages and validates the static HTML profile structurally. Further review proved and fixed comment/CDATA and duplicate-attribute HTML parser differences, missing PDF series legends, and clipped/unwrapped identity footers; measured layout checks fail explicitly on excess footer/legend content.
- The example produced an actual **10-page PDF**, continuous offline HTML and
  **10-slide native PPTX**. PDF text extraction retains supplied numbers,
  accented/Cyrillic names, comparison, limitations and provenance. Rendered
  PDF pages and native PPTX slides were visually inspected.
- Chromium loaded HTML offline with its embedded font at widths 1440, 768,
  390 and 320: no overflow, no page errors and **zero HTTP(S) requests**.
- LibreOfficeDev 26.8 opened and rendered the native PPTX to a 10-page PDF.
  A test edited native slide text and the linked chart cache/XLSX number
  (64 to 68), then opened/rendered/saved it through LibreOffice. Edited text
  and native chart data survived; all three native tables remained native.

This is structural, numerical-parity and real-renderer evidence, not a claim
of completed host application integration or human report approval. Interactive
Microsoft PowerPoint UI editing was not exercised. LibreOffice's own export
retains native chart data but removes the separate embedded XLSX; the original
SDK artifact contains the XLSX and correct internal relationships. Verify any
required PowerPoint/LibreOffice editing workflow against the target application's
version and install the named font to avoid fallback differences. Complex
script shaping is explicitly unavailable, and such text fails validation.

Rerun `report_render` to obtain artifacts and manifests; use `inspect_report`
for bounded structural inspection. Neither a manifest nor successful rendering
permits application adoption without the host's exact approval/source bindings.

## Section-first fixtures and historical compatibility

`tests/fixtures/section-first-report.json` exercises an opening narrative, long
chart labels, a missing observation, a dense native table, paired charts on
one slide, definitions and internal section and row links.
`tests/fixtures/dense-section-first-report.json` has a three-chart trend
section and a 24-row anchored table. Tests inspect the actual chart frames,
workbook values and gaps, table preview, complete notes and internal links.
The navigation fixture checks late appendix rows and destinations in PDF,
HTML and PowerPoint notes. A separate overflow case verifies the error for
source prose that exceeds one slide.

`make validate-pptx` renders both section-first fixtures alongside the
executive and navigation fixtures for Microsoft Open XML validation. The
hosted Rust workflow also generates all three default-template artifacts
from the pinned 0.4.0 source and checks that the candidate inspector accepts
the original bytes and manifests with a matching schema-1 typed digest.

To render the section-first fixture from the repository root:

```sh
cargo run --manifest-path office-sdk/Cargo.toml --no-default-features --features report --locked --example report_render -- office-sdk/tests/fixtures/section-first-report.json office-sdk/target/section-first section_first_v1
```

Desktop PowerPoint fit, editing and persistence remain visual acceptance
checks. The test suite checks the native parts, embedded cells and workbook
relationships before a host opens or edits the files.
