# Stable dependency and security rollup

Reviewed 2026-10-21 from exact fetched main `65cb029a500a1f0f915f49087799a8966bb9e52a`.

The paginated GitHub inventory returned zero open pull requests and zero open code-scanning alerts. That is an inventory result, not a clean security scan. No alerts were dismissed. This repository ships two Rust application packages and two independent C++ samples; no container base, native download pin or pre-existing Actions workflow was present. The C++ samples use the platform compiler and standard library, with no fetched third-party dependency.

Both Rust manifests now require the verified latest stable direct/development crate releases below. Versions were checked against the authoritative [crates.io sparse index](https://index.crates.io/config.json), excluding yanked/prerelease entries. Both binary packages now commit their complete Cargo lockfiles. Rust builds use **1.99.1**, verified against the [official stable manifest](https://static.rust-lang.org/dist/channel-rust-stable.toml); the declared consumer MSRV remains **1.95.0**, tested separately. Edition and package versions are unchanged.

| Crate | Latest stable | Source |
| --- | --- | --- |
| anyhow | 1.0.104 | [Index](https://index.crates.io/an/yh/anyhow) |
| base64 | 0.23.1 | [Index](https://index.crates.io/ba/se/base64) |
| bytes | 1.12.1 | [Index](https://index.crates.io/by/te/bytes) |
| chrono | 0.4.45 | [Index](https://index.crates.io/ch/ro/chrono) |
| clap | 4.6.7 | [Index](https://index.crates.io/cl/ap/clap) |
| colored | 3.1.1 | [Index](https://index.crates.io/co/lo/colored) |
| console | 0.16.6 | [Index](https://index.crates.io/co/ns/console) |
| dialoguer | 0.12.0 | [Index](https://index.crates.io/di/al/dialoguer) |
| dirs | 7.0.0 | [Index](https://index.crates.io/di/rs/dirs) |
| futures-util | 0.3.34 | [Index](https://index.crates.io/fu/tu/futures-util) |
| image | 0.25.10 | [Index](https://index.crates.io/im/ag/image) |
| indicatif | 0.18.6 | [Index](https://index.crates.io/in/di/indicatif) |
| lettre | 0.11.23 | [Index](https://index.crates.io/le/tt/lettre) |
| lopdf | 0.45.0 | [Index](https://index.crates.io/lo/pd/lopdf) |
| mail-parser | 0.11.9 | [Index](https://index.crates.io/ma/il/mail-parser) |
| md5 | 0.8.1 | [Index](https://index.crates.io/3/m/md5) |
| mime | 0.3.17 | [Index](https://index.crates.io/mi/me/mime) |
| pdf-writer | 0.15.0 | [Index](https://index.crates.io/pd/f-/pdf-writer) |
| printpdf | 0.12.8 | [Index](https://index.crates.io/pr/in/printpdf) |
| pulldown-cmark | 0.13.4 | [Index](https://index.crates.io/pu/ll/pulldown-cmark) |
| quick-xml | 0.42.0 | [Index](https://index.crates.io/qu/ic/quick-xml) |
| regex | 1.13.1 | [Index](https://index.crates.io/re/ge/regex) |
| serde | 1.0.229 | [Index](https://index.crates.io/se/rd/serde) |
| serde-xml-rs | 0.8.2 | [Index](https://index.crates.io/se/rd/serde-xml-rs) |
| serde_json | 1.0.154 | [Index](https://index.crates.io/se/rd/serde_json) |
| tempfile | 3.27.0 | [Index](https://index.crates.io/te/mp/tempfile) |
| thiserror | 2.0.21 | [Index](https://index.crates.io/th/is/thiserror) |
| time | 0.3.55 | [Index](https://index.crates.io/ti/me/time) |
| tokio | 1.54.0 | [Index](https://index.crates.io/to/ki/tokio) |
| uuid | 1.28.0 | [Index](https://index.crates.io/uu/id/uuid) |
| walkdir | 2.5.0 | [Index](https://index.crates.io/wa/lk/walkdir) |
| warp | 0.4.3 | [Index](https://index.crates.io/wa/rp/warp) |
| zip | 9.0.5 | [Index](https://index.crates.io/3/z/zip) |

## Independent baseline dependency audit

Because upstream main did not commit lockfiles, a fresh resolution of each unchanged baseline manifest was generated in isolated temporary directories. These are reproducible baseline resolution findings, not claims about a previously published binary or historical deployment. Each vulnerable component is a TP at the dependency version level; application reachability is stated separately.

| Finding | Baseline scope and why | Implemented fix and validation plan |
| --- | --- | --- |
| [RUSTSEC-2026-0258](https://rustsec.org/advisories/RUSTSEC-2026-0258.html), h2 unbounded empty DATA frames | TP in both packages: warp 0.3 resolves h2 0.3.27; patched releases require h2 >= 0.4.16. The sample servers accept HTTP input on loopback. | Migrate to warp 0.4.3 with explicit server/multipart/websocket features, resolving patched h2 0.4; compile and test every server binary and verify both lockfile audits. |
| [RUSTSEC-2026-0367](https://rustsec.org/advisories/RUSTSEC-2026-0367.html), ZIP SUID/SGID permissions | TP component in Office SDK: ZIP 0.6.6 is in the affected range. SDK code reads ZIP entries and does not call ZIP's filesystem extraction API, so no direct exploitation path is established. | Upgrade ZIP to 9.0.5 (patched >= 9); migrate writer options and fallible entry names, preserve Office archive round trips, and verify the final lock audit. |
| [RUSTSEC-2026-0368](https://rustsec.org/advisories/RUSTSEC-2026-0368.html), escaping ZIP symlink target | TP component in Office SDK: same vulnerable ZIP version. Filesystem extraction API is not called by the SDK. | Same implemented ZIP update; retain explicit extraction reachability limitation rather than misclassifying the component detection as FP. |
| [RUSTSEC-2026-0369](https://rustsec.org/advisories/RUSTSEC-2026-0369.html), permissions through ZIP symlinks | TP component in Office SDK: same vulnerable ZIP version. Filesystem extraction API is not called by the SDK. | Same implemented ZIP update and complete Office format tests. |

The updated lockfiles have **zero known RustSec vulnerabilities** at review time. The Office SDK retains the visible informational [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html) maintenance warning for `paste` 1.0.15 through current `image` AVIF/SIMD dependencies (`rav1e` and `pulp`). This is an upstream maintenance concern without a listed security vulnerability or patched version. No advisory ignore, dependency feature removal or local package substitution hides it. Follow maintained upstream image/rav1e/pulp releases to remove this warning when supported.

## Validation

The root `make ci` checks formatting and every target/feature, runs unit, integration, example and documentation tests, builds docs, audits both complete lockfiles, and checks every target on Rust 1.95.0. A pinned Actions workflow runs this complete gate once on Linux. No hooks or local gate existed previously; no hooks are bypassed.

ZIP 9 now returns fallible entry names and typed writer options. Office entry enumeration propagates errors rather than panicking, preserving the public error type. Major updates preserve document formats, server features and existing examples. Final executed checks and any limitations are recorded in the PR; a dependency scan alone does not validate parser behavior or prove absence of application vulnerabilities.

## Full-gate repair, 2026-10-28

The original dependency-rollup head `aaf882b7a6d940f5434769a15bad617bf5a84b16` reproduced the 13 missing XLSX methods recorded in [baseline compiler evidence](baseline-check-evidence.txt). Completing the existing XLSX state APIs exposed the previously unreachable integration suite. This follow-up preserves every original assertion and fixes the reproduced failures:

- Add XLSX metadata/page-setup setters, chart/table collection methods, text extraction, VBA accessors, and statistics. Save/load now preserves escaped metadata and binary VBA, with macro-enabled content types and a VBA relationship.
- Use XLSX page enums in the XLSX integration test and derive equality for PDF annotation types.
- Generate complete PNG/JPEG fixtures in the tests that require valid images. The PDF comprehensive fixture now creates the five bookmarks its existing assertion expects.
- Decode PDF metadata using lopdf's PDF text-string decoder, including UTF-16, and include table-cell text in PDF statistics.
- Preserve PPTX metadata and binary VBA, native media parts and speaker notes. Preserve SDK animation definitions in a namespaced OOXML presentation extension, including after repeated save/load cycles.

The new regressions inspect actual archive parts, VBA relationships/content types, XML escaping, Unicode PDF metadata, image bytes/dimensions, notes, and animation parameters across two saves. The test target runs doctests only for the SDK library, because the API sample is binary-only. The hosted workflow runs `make ci` once; `make ci-focused` remains available as an optional developer command. No target, assertion, audit or MSRV check is skipped.

These repairs do not establish advanced rendering support that the original SDK model did not implement. XLSX chart/table collections and page setup remain in-memory state; this change does not implement their drawing/table/print-layout export. The PPTX extension retains SDK animation definitions, without generating native PowerPoint timing trees or claiming animation playback. No renderer, deployment or application-security acceptance is claimed by the Rust build and tests.

The original dependency audit evidence remains historical, dated 2026-10-21. Current Linux validation and hosted check receipts are recorded in the PR follow-up.

The current audit additionally detected three vulnerable locked packages: [ctutils RUSTSEC-2026-0385](https://rustsec.org/advisories/RUSTSEC-2026-0385.html), [pulldown-cmark RUSTSEC-2026-0414](https://github.com/RustSec/advisory-db/blob/main/crates/pulldown-cmark/RUSTSEC-2026-0414.md), and [tokio RUSTSEC-2026-0384](https://rustsec.org/advisories/RUSTSEC-2026-0384.html). The follow-up updates ctutils 0.4.3 to 0.4.4, pulldown-cmark 0.13.4 to 0.13.5, and tokio 1.54.0 to 1.54.1 in both applications. These are component-version findings; exploitability of the application is not asserted. Features and the Rust 1.95 MSRV are preserved. The original security evidence above remains a dated record, rather than a claim that its versions are still current.
Current local result on Ubuntu x86_64 (`cmsec-host-02`, two Cargo jobs): `make ci` passed with **131 tests**, including 95 integration tests and six dependency/archive regressions; formatting, every target/feature, SDK doctests, rustdoc and both Rust 1.95 all-target checks passed. Clippy passed for all targets in both packages with existing warnings. Both complete lockfile audits report **zero vulnerabilities**, using advisory database commit `40f5be6fb12ed93fc591bf4ab23b7d793b3f2239`; the `paste` maintenance warning remains visible.

Lockfile SHA-256: Office SDK `af3b19023d47e726c81389fc2c60dd6e93389ca70888247d57355cb04ae24fb1`; API sample `500a47248555a4e5f5c359ecf02bb178c77c670d4d7d072a56d69d8e8c198cac`.
