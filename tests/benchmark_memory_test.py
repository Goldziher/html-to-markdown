from __future__ import annotations

import gc
import tracemalloc
from contextlib import contextmanager
from typing import TYPE_CHECKING, Any, cast

if TYPE_CHECKING:
    from collections.abc import Generator
    from pathlib import Path

import pytest

from html_to_markdown import convert_to_markdown

pytest_plugins: list[str] = []

try:
    import psutil
except ImportError:
    pytest.skip("psutil not available", allow_module_level=True)

try:
    import memray

    MEMRAY_AVAILABLE = True
except ImportError:
    MEMRAY_AVAILABLE = False

try:
    from .performance_test import generate_complex_html
except ImportError:
    from tests.performance_test import generate_complex_html


@contextmanager
def memory_snapshot() -> Generator[dict[str, Any], None, None]:
    tracemalloc.start()
    gc.collect()

    snapshot_before = tracemalloc.take_snapshot()
    initial_stats = snapshot_before.statistics("lineno")

    process = psutil.Process()
    process_info = {"rss_before": process.memory_info().rss}

    memory_data = {
        "tracemalloc_before": initial_stats,
        "process_before": process_info,
        "tracemalloc_after": None,
        "process_after": {},
        "peak_memory": 0,
        "allocations_diff": [],
    }

    try:
        yield memory_data
    finally:
        snapshot_after = tracemalloc.take_snapshot()
        final_stats = snapshot_after.statistics("lineno")

        process = psutil.Process()
        memory_data["process_after"] = {"rss_after": process.memory_info().rss}
        memory_data["peak_memory"] = process.memory_info().rss

        memory_data["tracemalloc_after"] = final_stats

        top_stats = snapshot_after.compare_to(snapshot_before, "lineno")
        memory_data["allocations_diff"] = cast(
            "Any",
            [
                {"filename": stat.traceback.format()[0], "size_diff": stat.size_diff, "count_diff": stat.count_diff}
                for stat in top_stats[:10]
            ],
        )

        tracemalloc.stop()


class TestMemoryProfiling:
    def test_memory_baseline_small(self) -> None:
        html = generate_complex_html(size_factor=5)

        with memory_snapshot() as memory_data:
            result = convert_to_markdown(html)

        assert len(result) > 0

        memory_used_mb = (
            (memory_data["process_after"]["rss_after"] - memory_data["process_before"]["rss_before"]) / 1024 / 1024
        )

        assert memory_used_mb < 50, f"Small document used {memory_used_mb:.2f}MB"

    def test_memory_baseline_large(self) -> None:
        html = generate_complex_html(size_factor=100)

        with memory_snapshot() as memory_data:
            result = convert_to_markdown(html)

        assert len(result) > 0

        memory_used_mb = (
            (memory_data["process_after"]["rss_after"] - memory_data["process_before"]["rss_before"]) / 1024 / 1024
        )

        assert memory_used_mb < 200, f"Large document used {memory_used_mb:.2f}MB"

    def test_memory_leak_detection(self) -> None:
        html = generate_complex_html(size_factor=20)

        memory_usage = []

        for _i in range(5):
            process = psutil.Process()
            _memory_before = process.memory_info().rss

            for _ in range(10):
                result = convert_to_markdown(html)
                assert len(result) > 0

            gc.collect()

            memory_after = process.memory_info().rss
            memory_usage.append(memory_after)

        if len(memory_usage) >= 3:
            growth_rate = (memory_usage[-1] - memory_usage[0]) / len(memory_usage)
            max_acceptable_growth = 1024 * 1024

            assert growth_rate < max_acceptable_growth, (
                f"Potential memory leak detected: {growth_rate / 1024 / 1024:.2f}MB growth per iteration"
            )


@pytest.mark.skipif(not MEMRAY_AVAILABLE, reason="memray not available on this platform")
class TestMemrayProfiling:
    def test_memray_profile_conversion(self, tmp_path: Path) -> None:
        html = generate_complex_html(size_factor=50)
        output_file = tmp_path / "memray_profile.bin"

        with memray.Tracker(output_file):
            result = convert_to_markdown(html)

        assert len(result) > 0
        assert output_file.exists()


def run_memory_analysis() -> None:
    print("🧠 Running Memory Analysis")
    print("=" * 40)

    sizes = [10, 25, 50, 100]

    for size in sizes:
        html = generate_complex_html(size_factor=size)
        input_size_mb = len(html) / 1024 / 1024

        print(f"\n📊 Document size factor: {size} ({input_size_mb:.2f}MB)")

        with memory_snapshot() as memory_data:
            result = convert_to_markdown(html)

        memory_used_mb = (
            (memory_data["process_after"]["rss_after"] - memory_data["process_before"]["rss_before"]) / 1024 / 1024
        )

        efficiency = len(result) / (memory_used_mb * 1024 * 1024) if memory_used_mb > 0 else float("inf")

        print(f"   Memory used: {memory_used_mb:.2f}MB")
        print(f"   Output size: {len(result) / 1024:.2f}KB")
        print(f"   Efficiency: {efficiency:.2f} chars/byte")

        if memory_data["allocations_diff"]:
            print("   Top allocations:")
            for allocation in memory_data["allocations_diff"][:3]:
                if allocation["size_diff"] > 0:
                    print(f"     {allocation['filename']}: +{allocation['size_diff'] / 1024:.2f}KB")


if __name__ == "__main__":
    run_memory_analysis()
