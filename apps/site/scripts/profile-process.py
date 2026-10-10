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
            rss = next((
                int(line.split()[1]) for line in status if line.startswith("VmRSS:")
            ), 0)
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


def tree(snapshot, root, starts, owner):
    # Track identities across setsid/reparenting, and include adopted children
    # even if their former parent exited between samples. This standalone
    # subreaper starts only the profiled command, so its children belong to it.
    selected = {
        pid for pid, start in starts.items()
        if pid in snapshot and snapshot[pid]["start"] == start
    }
    selected.update(pid for pid, value in snapshot.items() if value["ppid"] == owner)
    if root in snapshot and snapshot[root]["start"] == starts.get(root):
        selected.update(pid for pid, value in snapshot.items() if value["group"] == root)
    while True:
        descendants = {
            pid for pid, value in snapshot.items() if value["ppid"] in selected
        }
        expanded = selected | descendants
        if expanded == selected:
            return {pid: snapshot[pid] for pid in selected if pid in snapshot}
        selected = expanded


def signal_processes(selected, kind):
    # The pidfd pins the selected process, so PID reuse between validation and
    # signal delivery cannot terminate a different process.
    for pid, value in selected.items():
        try:
            descriptor = os.pidfd_open(pid)
        except ProcessLookupError:
            continue
        try:
            try:
                start = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[19]
            except (OSError, IndexError):
                continue
            if start == value["start"]:
                try:
                    signal.pidfd_send_signal(descriptor, kind)
                except ProcessLookupError:
                    pass
        finally:
            os.close(descriptor)


def reap_adopted(snapshot, owner, root):
    for pid, value in snapshot.items():
        if value["ppid"] == owner and pid != root:
            try:
                os.waitpid(pid, os.WNOHANG)
            except ChildProcessError:
                pass


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
    owner = os.getpid()
    peak = 0
    per_pid = {}
    starts = {}
    initial = processes()
    if child.pid in initial:
        starts[child.pid] = initial[child.pid]["start"]
    samples = 0
    stopped = None
    try:
        while True:
            selected = tree(processes(), child.pid, starts, owner)
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
            if child.poll() is not None:
                break
            if time.monotonic() - started > args.timeout:
                stopped = "deadline"
                break
            time.sleep(0.05)
    except KeyboardInterrupt:
        stopped = "interrupted"
    finally:
        cleanup_started = time.monotonic()
        cleanup_identities = set()
        remaining = {}
        while True:
            child.poll()
            snapshot = processes()
            reap_adopted(snapshot, owner, child.pid)
            remaining = tree(processes(), child.pid, starts, owner)
            if not remaining:
                break
            cleanup_identities.update((pid, value["start"]) for pid, value in remaining.items())
            starts.update((pid, value["start"]) for pid, value in remaining.items())
            elapsed_cleanup = time.monotonic() - cleanup_started
            signal_processes(remaining, signal.SIGKILL if elapsed_cleanup >= 5 else signal.SIGTERM)
            if elapsed_cleanup >= 6:
                stopped = stopped or "cleanup incomplete"
                break
            time.sleep(0.01)
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
            "cleanupProcessCount": len(cleanup_identities),
            "remainingProcessCount": len(remaining),
        }
        report_path.write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report), flush=True)
    return 0 if child.returncode == 0 and stopped is None else 1


if __name__ == "__main__":
    raise SystemExit(main())
