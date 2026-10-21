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

The root `make ci` checks formatting and every target/feature, runs unit, integration, example and documentation tests, builds docs, audits both complete lockfiles, and checks every target on Rust 1.95.0. A pinned Actions workflow runs the focused migration checks and then the full gate on Linux. No hooks or local gate existed previously; no hooks are bypassed.

ZIP 9 now returns fallible entry names and typed writer options. Office entry enumeration propagates errors rather than panicking, preserving the public error type. Major updates preserve document formats, server features and existing examples. Final executed checks and any limitations are recorded in the PR; a dependency scan alone does not validate parser behavior or prove absence of application vulnerabilities.

## Pre-existing full-gate blocker

The exact baseline revision was exported into an isolated temporary directory and checked with its unchanged manifest and a freshly resolved lockfile. `cargo check --all-targets --all-features --locked` exits 101: `advanced_xlsx_simple` calls 13 missing methods, including `set_author`, `set_page_size`, `add_chart`, `add_vba_macro` and `get_document_stats`. The same methods are absent from the original `src/xlsx.rs`. The updated all-target check reproduces these errors and also exposes the existing monolithic integration test's missing XLSX calls and missing PDF `AnnotationType: PartialEq`. See [baseline compiler evidence](baseline-check-evidence.txt).

The full `make ci` and new hosted gate retain these failures visibly. They are not skipped or claimed to pass. `make ci-focused` separately validates the library and every shipped binary, dedicated dependency regression tests, library/doctests, the API test application, audits and preserved MSRV. This rollup remains a draft pending an independent implementation of the incomplete XLSX APIs and corresponding old test repairs; it does not invent Office features or remove test assertions to manufacture a green full gate.

## Executed results

`CARGO_BUILD_JOBS=2 make ci-focused` passed on the final lockfiles: all Office library/binary builds, all API-test targets, **25 library tests**, **2 cross-format/archive regressions**, and **2 HTTP health tests** (one in each application). Doctests, rustdoc, both complete lockfile audits and separate **Rust 1.95.0 compilation checks** for the library, binaries, dedicated regression target and API-test targets passed. No existing test is ignored or weakened by this change. `cargo clippy` passed for the Office library/binaries and every API-test target with existing warnings; warning-free status is not claimed. `actionlint` and `git diff --check` passed.

The final `make ci` still exits 2 at its all-target check with the same 13 missing XLSX methods reproduced on the exact baseline. One unrelated example's pre-existing move-out-of-borrow compile error was repaired using the document-format enum's existing `Clone` implementation. The broad XLSX API gap and the old monolithic integration test remain visible blockers. Linux hosted validation remains pending; local execution used macOS ARM64 and no published artifact/security scan is claimed. [Structured audit evidence](dependency-audit-evidence.json) includes baseline advisory records, final lockfile hashes and final zero-vulnerability audits, retaining the maintenance warning.
