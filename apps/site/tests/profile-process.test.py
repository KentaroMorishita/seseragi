"""Linux process-tree bounds, session detachment, PID identity and cleanup."""

import importlib.util
import json
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/profile-process.py"
spec = importlib.util.spec_from_file_location("profile_process", SCRIPT)
monitor = importlib.util.module_from_spec(spec)
spec.loader.exec_module(monitor)


@unittest.skipUnless(
    sys.platform == "linux",
    "Linux process-tree instrumentation requires /proc and pidfd; general site gates remain enabled",
)
class ProcessBounds(unittest.TestCase):
    def run_monitor(self, source, limit=64, timeout=3):
        with tempfile.TemporaryDirectory(prefix="seseragi-profile-test-") as directory:
            report = Path(directory) / "report.json"
            pid_file = Path(directory) / "orphan.json"
            result = subprocess.run(
                [sys.executable, str(SCRIPT), "--report", str(report),
                 "--limit-mib", str(limit), "--timeout", str(timeout), "--",
                 sys.executable, "-c", source, str(pid_file)],
                capture_output=True, text=True, timeout=10,
            )
            self.assertTrue(report.exists(), result.stderr)
            evidence = json.loads(report.read_text())
            if pid_file.exists():
                identity = json.loads(pid_file.read_text())
                current = monitor.processes().get(identity["pid"])
                self.assertTrue(current is None or current["start"] != identity["start"],
                                "adopted orphan must be terminated and reaped")
            self.assertEqual(evidence["remainingProcessCount"], 0)
            return result, evidence

    def test_normal_descendant_is_sampled(self):
        result, evidence = self.run_monitor("""
import os, time
if os.fork() == 0:
    memory = bytearray(8 * 1024 * 1024)
    time.sleep(0.2)
    os._exit(0)
os.wait()
""")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIsNone(evidence["stopReason"])
        self.assertGreaterEqual(len(evidence["perPidPeakRssKiB"]), 2)
        self.assertGreater(evidence["peakTreeRssKiB"], 16 * 1024)

    def test_deadline_stops_and_reaps_root(self):
        result, evidence = self.run_monitor("import time; time.sleep(2)", timeout=0.1)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(evidence["stopReason"], "deadline")
        self.assertGreaterEqual(evidence["cleanupProcessCount"], 1)

    def test_detached_orphan_still_triggers_memory_bound(self):
        # Allocate only 64 MiB, after the intermediate parent exits and the
        # grandchild creates a separate session. Root stays alive throughout.
        result, evidence = self.run_monitor("""
import json, os, sys, time
from pathlib import Path
if os.fork() == 0:
    os.setsid()
    if os.fork() == 0:
        start = Path('/proc/self/stat').read_text().rsplit(')', 1)[1].split()[19]
        Path(sys.argv[1]).write_text(json.dumps({'pid': os.getpid(), 'start': start}))
        time.sleep(0.15)
        memory = bytearray(64 * 1024 * 1024)
        time.sleep(2)
    os._exit(0)
time.sleep(1)
""", limit=48)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(evidence["stopReason"], "memory limit")
        self.assertGreater(evidence["peakTreeRssKiB"], 48 * 1024)
        self.assertGreaterEqual(evidence["cleanupProcessCount"], 2)

    def test_identity_seeds_expand_tracked_and_adopted_trees(self):
        def process(parent, group, start):
            return {"ppid": parent, "group": group, "start": start, "rssKiB": 1}
        snapshot = {
            11: process(90, 11, "root"),
            22: process(800, 22, "reused"),
            23: process(800, 23, "tracked"),
            24: process(23, 24, "child"),
            25: process(90, 25, "adopted"),
            26: process(25, 26, "adopted-child"),
        }
        selected = monitor.tree(snapshot, 11, {11: "root", 22: "old", 23: "tracked"}, 90)
        self.assertEqual(set(selected), {11, 23, 24, 25, 26})
        # A recycled root PID must not seed its former process group either.
        snapshot[11] = process(800, 11, "reused-root")
        snapshot[27] = process(800, 11, "unrelated")
        self.assertEqual(set(monitor.tree(snapshot, 11, {11: "root"}, 90)), {25, 26})

    def test_cleanup_does_not_signal_reused_pid(self):
        current = ") " + " ".join(["0"] * 19 + ["new-start"])
        with patch.object(monitor.os, "pidfd_open", return_value=500), \
             patch.object(monitor.os, "close") as close, \
             patch.object(monitor.Path, "read_text", return_value=current), \
             patch.object(monitor.signal, "pidfd_send_signal") as send:
            monitor.signal_processes({42: {"start": "old-start"}}, signal.SIGKILL)
            send.assert_not_called()
            close.assert_called_once_with(500)


if __name__ == "__main__":
    unittest.main()
