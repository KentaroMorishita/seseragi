"""Bound a Linux command and sample its process tree, including fresh children."""

import argparse
import ctypes
import json
import os
from pathlib import Path
import signal
import subprocess
import time


def processes():
    result = {}
    for entry in os.scandir("/proc"):
        if not entry.name.isdigit():
            continue
        try:
            stat = Path(entry.path, "stat").read_text().rsplit(")", 1)[1].split()
            status = Path(entry.path, "status").read_text().splitlines()
            rss = next(
                int(line.split()[1]) for line in status if line.startswith("VmRSS:")
            )
            result[int(entry.name)] = {
                "ppid": int(stat[1]),
                "group": int(stat[2]),
                "rssKiB": rss,
                "start": stat[19],
            }
        except (OSError, ValueError, StopIteration):
            # A process can exit between enumeration and either read.
            continue
    return result


def tree(snapshot, root):
    # Group membership also catches children orphaned after a failed build.
    selected = {pid for pid, value in snapshot.items() if value["group"] == root}
    selected.add(root)
    while True:
        descendants = {
            pid for pid, value in snapshot.items() if value["ppid"] in selected
        }
        expanded = selected | descendants
        if expanded == selected:
            return {pid: snapshot[pid] for pid in selected if pid in snapshot}
        selected = expanded


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", required=True)
    parser.add_argument("--timeout", type=float, required=True)
    parser.add_argument("--limit-mib", type=int, default=4096)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command or args.timeout <= 0 or args.limit_mib <= 0:
        parser.error("a command and positive bounds are required")
    if not Path("/proc/self/stat").exists():
        parser.error("process-tree RSS profiling requires Linux /proc")
    report_path = Path(args.report)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    # Reap descendants if a timed-out host exits before its renderer/browser.
    if ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0) != 0:
        raise OSError(ctypes.get_errno(), "cannot become a child subreaper")
    started = time.monotonic()
    child = subprocess.Popen(command, start_new_session=True)
    peak = 0
    per_pid = {}
    starts = {}
    samples = 0
    stopped = None
    try:
        while child.poll() is None:
            selected = tree(processes(), child.pid)
            observed = sum(value["rssKiB"] for value in selected.values())
            peak = max(peak, observed)
            samples += 1
            for pid, value in selected.items():
                key = str(pid)
                per_pid[key] = max(per_pid.get(key, 0), value["rssKiB"])
                starts[pid] = value["start"]
            if observed > args.limit_mib * 1024:
                stopped = "memory limit"
                break
            if time.monotonic() - started > args.timeout:
                stopped = "deadline"
                break
            time.sleep(0.05)
    except KeyboardInterrupt:
        stopped = "interrupted"
    finally:
        snapshot = processes()
        remaining = tree(snapshot, child.pid)
        remaining.update({
            pid: snapshot[pid] for pid, start in starts.items()
            if pid in snapshot and snapshot[pid]["start"] == start
        })
        if remaining:
            # Only this command's group and current descendants are terminated.
            for pid in remaining:
                try:
                    os.kill(pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                pass
            for pid in remaining:
                try:
                    os.kill(pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
        child.wait()
        while True:
            try:
                pid, _ = os.waitpid(-1, os.WNOHANG)
                if pid == 0:
                    break
            except ChildProcessError:
                break
        report = {
            "command": command,
            "elapsedMs": round((time.monotonic() - started) * 1000),
            "exitCode": child.returncode,
            "stopReason": stopped,
            "peakTreeRssKiB": peak,
            "perPidPeakRssKiB": per_pid,
            "sampleIntervalMs": 50,
            "samples": samples,
            "limitMiB": args.limit_mib,
            "deadlineSeconds": args.timeout,
            "cleanupProcessCount": len(remaining),
        }
        report_path.write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report), flush=True)
    return 0 if child.returncode == 0 and stopped is None else 1


if __name__ == "__main__":
    raise SystemExit(main())
