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
