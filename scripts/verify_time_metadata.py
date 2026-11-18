"""Hosted-only before/after proof for metadata that must not silently become now."""

import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BEFORE_SHA = "be6978105ae333b507a590df6b0c1ca5969a1268"
TEST = Path("office-sdk/tests/maintained_time.rs")
NAMES = (
    "present_unsupported_spreadsheet_dates_are_rejected",
    "present_unsupported_presentation_dates_are_rejected",
)


def head(path):
    return subprocess.check_output(
        ["git", "-C", str(path), "rev-parse", "HEAD"], text=True
    ).strip()


def main():
    before = ROOT / "before"
    candidate = os.environ["CANDIDATE_SHA"]
    if head(ROOT) != candidate or head(before) != BEFORE_SHA:
        raise RuntimeError(
            "Timestamp proof checkouts differ from the exact source commits"
        )
    evidence = ROOT / "target/time-metadata-proof"
    evidence.mkdir(parents=True, exist_ok=False)
    test_bytes = (ROOT / TEST).read_bytes()
    shutil.copyfile(ROOT / TEST, before / TEST)
    targets = Path(os.environ["RUNNER_TEMP"]) / "office-metadata-proof"
    results = []
    for label, checkout in (("before", before), ("after", ROOT)):
        target = targets / label
        environment = dict(
            os.environ, CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS="2"
        )
        command = [
            "cargo",
            "+1.100.0",
            "test",
            "--manifest-path",
            str(checkout / "office-sdk/Cargo.toml"),
            "--test",
            "maintained_time",
            "--no-default-features",
            "--features",
            "report",
            "--locked",
            "--",
            "present_unsupported_",
            "--nocapture",
        ]
        result = subprocess.run(
            command,
            cwd=checkout,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
        )
        log = result.stdout + result.stderr
        (evidence / f"{label}.log").write_text(log)
        expected_code = 101 if label == "before" else 0
        marker = "FAILED" if label == "before" else "ok"
        count = "2 failed" if label == "before" else "2 passed"
        if (
            result.returncode != expected_code
            or count not in log
            or any(f"test {name} ... {marker}" not in log for name in NAMES)
        ):
            raise RuntimeError(
                f"{label} did not execute both timestamp regressions as expected; see its log"
            )
        results.append(
            {
                "label": label,
                "source_sha": BEFORE_SHA if label == "before" else candidate,
                "target_directory": str(target),
                "exit_code": result.returncode,
                "regressions": list(NAMES),
                "transcript_sha256": hashlib.sha256(log.encode()).hexdigest(),
            }
        )
    (evidence / "proof.json").write_text(
        json.dumps(
            {
                "test_source_sha256": hashlib.sha256(test_bytes).hexdigest(),
                "test_source_identical": test_bytes == (before / TEST).read_bytes(),
                "isolated_target_directories": True,
                "runs": results,
            },
            indent=2,
        )
        + "\n"
    )
    print(
        "Both real document parsers reproduce silent replacement before and reject unsupported dates after."
    )


if __name__ == "__main__":
    main()
