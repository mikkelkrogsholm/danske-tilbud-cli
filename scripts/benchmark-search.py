#!/usr/bin/env python3
"""Measure one local search against an owned, synthetic JSONL snapshot."""
import json
import pathlib
import platform
import shlex
import statistics
import subprocess
import sys
import tempfile
import time
from datetime import date, timedelta


def main():
    if len(sys.argv) != 2:
        sys.exit(f"Usage: {sys.argv[0]} /path/to/tilbud-release-binary")
    binary = pathlib.Path(sys.argv[1]).resolve(strict=True)
    example = (pathlib.Path(__file__).resolve().parent.parent / "docs/offer-v1.md").read_text()
    base = json.loads(example.split("```json\n")[1].split("\n```")[0])
    today = date.today()
    base.update(
        source="tjek",
        chain_id="netto",
        price_kind="offer",
        validity={
            "starts": {"precision": "date", "value": str(today - timedelta(days=365))},
            "ends": {"precision": "date", "value": str(today + timedelta(days=365))},
        },
    )
    manifest = {
        "record_type": "snapshot", "schema_version": 1, "source": "tjek",
        "fetched_at": "2026-11-02T09:00:00+01:00",
        "written_at": "2026-11-02T09:00:00+01:00",
        "coverage": "syntetisk eksempel", "complete": True, "offer_count": 12000,
    }
    headings = ("Økologisk letmælk", "Kaffe malet", "Smør saltet", "Sodavand citrus")
    descriptions = ("Maks. 6 stk. Pant tilkommer.", "Tilbud på udvalgte varer", "Kun med medlemskab")
    with tempfile.TemporaryDirectory(prefix="tilbud-search-benchmark-") as tmp:
        snapshot = pathlib.Path(tmp) / "tjek.jsonl"
        with snapshot.open("w", encoding="utf-8") as stream:
            stream.write(json.dumps(manifest, ensure_ascii=False) + "\n")
            for index in range(12000):
                row = dict(base, source_id=str(index), heading=headings[index % 4],
                           description=descriptions[index % 3],
                           source_reference=f"https://example.com/offers/{index}")
                stream.write(json.dumps(row, ensure_ascii=False) + "\n")
        command = [str(binary), "find", "kaffe", "--kilde", "tjek", "--data-dir", tmp]
        times = []
        for iteration in range(32):
            start = time.perf_counter()
            result = subprocess.run(command, check=True, stdout=subprocess.PIPE)
            elapsed = (time.perf_counter() - start) * 1000
            data = json.loads(result.stdout)["data"]
            if data["total"] != 3000 or len(data["results"]) != 20:
                raise ValueError("Fixture did not produce the expected 3,000 matches")
            if iteration >= 2:
                times.append(elapsed)
        print(f"machine: {platform.platform()} {platform.machine()} {platform.processor()}")
        print(f"command: {shlex.join(command)}")
        print(f"records=12000 input_bytes={snapshot.stat().st_size} output_bytes={len(result.stdout)}")
        print(f"warmups=2 n={len(times)} median_ms={statistics.median(times):.3f} "
              f"p95_ms={sorted(times)[28]:.3f}")


if __name__ == "__main__":
    main()
