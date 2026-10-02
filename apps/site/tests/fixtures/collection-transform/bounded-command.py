"""Bound a test child and its descendants; preserve its stdout/stderr bytes.

Linux RSS is sampled every 20 ms, so this is an observed resident-memory cap,
not an address-space reservation limit. JavaScript engines reserve large sparse
address spaces. A process group is killed on timeout, RSS, or output overrun.
"""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time

receipt, *argv = sys.argv[1:]
limit_seconds = 60
limit_rss = 1024 * 1024 * 1024
limit_output = 16 * 1024 * 1024
start = time.monotonic()
peak_rss = 0
reason = None


def resident_tree(pid):
    pending = [pid]
    seen = set()
    total = 0
    while pending:
        current = pending.pop()
        if current in seen:
            continue
        seen.add(current)
        try:
            status = Path(f"/proc/{current}/status").read_text()
            for line in status.splitlines():
                if line.startswith("VmRSS:"):
                    total += int(line.split()[1]) * 1024
            pending.extend(int(n) for n in Path(
                f"/proc/{current}/task/{current}/children"
            ).read_text().split())
        except (FileNotFoundError, ProcessLookupError):
            pass
    return total


if not Path("/proc/self/status").exists():
    raise SystemExit("Collection transform resource checks require Linux /proc")
with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
    process = subprocess.Popen(
        argv, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr,
        start_new_session=True,
    )
    while True:
        peak_rss = max(peak_rss, resident_tree(process.pid))
        elapsed = time.monotonic() - start
        if elapsed > limit_seconds:
            reason = "timeout"
        elif peak_rss > limit_rss:
            reason = "resident-memory-limit"
        elif max(os.fstat(stdout.fileno()).st_size,
                 os.fstat(stderr.fileno()).st_size) > limit_output:
            reason = "output-limit"
        if reason:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            break
        if process.poll() is not None:
            break
        time.sleep(0.02)
    Path(receipt).write_text(json.dumps({
        "argv": argv, "stdinClosedByHarness": True,
        "timeoutSeconds": limit_seconds, "residentMemoryLimitBytes": limit_rss,
        "residentMemoryScope": "sampled process tree RSS every 20 ms",
        "outputLimitBytesPerStream": limit_output,
        "peakObservedResidentBytes": peak_rss,
        "elapsedSeconds": time.monotonic() - start,
        "terminationReason": reason, "exitStatus": process.returncode,
    }, indent=2) + "\n")
    stdout.seek(0)
    stderr.seek(0)
    sys.stdout.buffer.write(stdout.read(limit_output))
    sys.stderr.buffer.write(stderr.read(limit_output))
    sys.stdout.buffer.flush()
    sys.stderr.buffer.flush()
raise SystemExit(124 if reason else process.returncode)
