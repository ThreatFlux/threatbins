# Migrating to Office SDK 0.4

Document dates, email dates and file/storage metadata now use the maintained
`office_sdk::Timestamp` re-export from Jiff 1.0 instead of Chrono's
`DateTime<Utc>`. This is a breaking Rust API change; use the new 0.4 release
and migrate all consumers together. No advisory exception is added.

```rust
# #[cfg(feature = "report")]
# fn example() -> Result<(), Box<dyn std::error::Error>> {
use office_sdk::Timestamp;
let created: Timestamp = "2026-11-18T12:00:00.123456789+02:00".parse()?;
assert_eq!(created.to_string(), "2026-11-18T10:00:00.123456789Z");
let now = Timestamp::now();
let from_unix = Timestamp::from_second(0)?;
# Ok(())
# }
```

Replace `Utc::now()` with `Timestamp::now()`, parse RFC 3339 input with
`text.parse::<Timestamp>()`, and handle errors from Unix timestamp constructors.
Use `strftime` for calendar formatting. Jiff's `as_second()` truncates negative
fractional Unix times toward zero, unlike Chrono's flooring `timestamp()`;
compare instants or use explicit rounding when that distinction matters.
Jiff normalizes leap-second input to second 59 rather than retaining a leap
second. Its supported calendar range is years -9999 through 9999.

File metadata JSON still contains RFC 3339 strings and nullable optional fields.
Offsets normalize to UTC and fractional seconds retain nanosecond precision.
Present spreadsheet/presentation dates outside the supported range, or malformed
dates, return an error before metadata can be replaced. Missing dates retain the
existing current-time default. Jiff's exact UTC timestamp bounds are narrower
than its civil calendar at the extreme endpoints; handle parse errors instead
of assuming every date in year 9999 is representable.
Spreadsheet/presentation XML keeps explicit `+00:00` timestamps. DOCX and email
headers keep their existing whole-second formatting. Parsed email dates that
exceed the supported timestamp range return an error instead of silently using
the current time; missing/unparseable mail dates retain the existing default.

The `report-model`, `report` and `full` feature sets remain available. Jiff is
optional and is enabled by `report`; `report-model` alone does not need it.
Existing Office-format, PDF and Microsoft schema validation remains in CI.
The old 0.3.1 archives and tags remain immutable; private consumers must wait
for the new producer archive and a separately qualified registry append.
