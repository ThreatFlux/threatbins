"""Regress unchanged-version reuse and real archive verification on GitHub."""

import hashlib
import io
import json
import tarfile
import tempfile
import unittest
from pathlib import Path

from office_release import reuse_existing, verify

SOURCE = "a" * 40
NEW_SOURCE = "b" * 40


class OfficeReleaseTests(unittest.TestCase):
    def test_config_only_commit_reuses_existing_version(self):
        self.assertTrue(reuse_existing(NEW_SOURCE, SOURCE, False))

    def test_changed_package_requires_a_version_bump(self):
        with self.assertRaises(ValueError):
            reuse_existing(NEW_SOURCE, SOURCE, True)

    def test_new_and_same_source_releases_can_publish_or_resume(self):
        self.assertFalse(reuse_existing(SOURCE, None, True))
        self.assertFalse(reuse_existing(SOURCE, SOURCE, False))

    def test_abbreviated_source_is_rejected(self):
        with self.assertRaises(ValueError):
            reuse_existing("abcdef0", SOURCE, False)

    def archive(self, directory, source=SOURCE, dirty=False):
        name = "office-sdk-0.3.0.crate"
        path = directory / name
        contents = {
            ".cargo_vcs_info.json": json.dumps(
                {"git": {"sha1": source, "dirty": dirty}}
            ).encode(),
            "Cargo.toml": b'[package]\nname="office-sdk"\nversion="0.3.0"\n',
        }
        with tarfile.open(path, "w:gz") as archive:
            for member, data in contents.items():
                info = tarfile.TarInfo("office-sdk-0.3.0/" + member)
                info.size = len(data)
                archive.addfile(info, io.BytesIO(data))
        checksum = hashlib.sha256(path.read_bytes()).hexdigest()
        (directory / (name + ".sha256")).write_text(checksum + "  " + name + "\n")
        return path, checksum

    def test_downloaded_archive_source_and_bytes_are_verified(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            _, checksum = self.archive(directory)
            self.assertEqual(verify(directory, "0.3.0", SOURCE), checksum)

    def test_archive_with_different_vcs_source_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.archive(directory, source=NEW_SOURCE)
            with self.assertRaises(ValueError):
                verify(directory, "0.3.0", SOURCE)

    def test_archive_from_dirty_source_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.archive(directory, dirty=True)
            with self.assertRaises(ValueError):
                verify(directory, "0.3.0", SOURCE)

    def test_changed_archive_is_rejected_even_with_matching_vcs(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            path, _ = self.archive(directory)
            path.write_bytes(path.read_bytes() + b"changed")
            with self.assertRaises(ValueError):
                verify(directory, "0.3.0", SOURCE)


if __name__ == "__main__":
    unittest.main()
