#!/usr/bin/env python3
"""Run Cargo's all-target selection in bounded batches, with both CI seeds.

Keep --workspace on every invocation: selecting one package changes feature
unification. Libraries therefore form one batch; named targets form one batch
per name (including names shared by multiple workspace members).
"""

import hashlib
import json
import os
from pathlib import Path
import re
import resource
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import threading
import tomllib

SEEDS = ("5350648285461741569", "5350648285461741570")
TARGET_LIMIT = 1024**3
FREE_MINIMUM = 5 * 1024**3
SUMMARY = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; "
    r"(\d+) ignored; (\d+) measured; (\d+) filtered out;.*$"
)
LIB_KINDS = {"lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"}


def target_key(package_id, target):
    return (package_id, tuple(target["kind"]), target["name"])


def target_options(package, target):
    """Metadata omits harness and bench; read only those manifest settings."""
    with open(package["manifest_path"], "rb") as source:
        manifest = tomllib.load(source)
    kind = target["kind"][0]
    if kind in LIB_KINDS:
        return manifest.get("lib", {})
    for entry in manifest.get(kind, []):
        if entry.get("name") == target["name"]:
            return entry
    return {}


def batches(metadata):
    members = set(metadata["workspace_members"])
    groups = {}
    skipped = []
    for package in metadata["packages"]:
        if package["id"] not in members:
            continue
        features = set(package["features"])
        for target in package["targets"]:
            kinds = set(target["kind"])
            if kinds == {"custom-build"}:
                continue  # Cargo owns build scripts as dependencies, never test roots.
            options = target_options(package, target)
            kind = target["kind"][0]
            if kinds & LIB_KINDS:
                selector = ("--lib",)
            elif kind in {"bin", "example", "test", "bench"}:
                selector = ("--" + kind, target["name"])
            else:
                raise RuntimeError(f"unsupported target kind: {target}")
            # --all-targets always selects lib/bin/example. Its --tests/--benches
            # arms select targets with test=true OR bench=true, respectively.
            if kind in {"test", "bench"} and not (
                target["test"] or options.get("bench", kind == "bench")
            ):
                skipped.append((target_key(package["id"], target), "test and bench disabled"))
                continue
            missing = set(target.get("required-features", [])) - features
            if kinds & LIB_KINDS:
                missing = set()  # Cargo never gates a library on required-features.
            if any("/" in feature or feature.startswith("dep:") for feature in missing):
                raise RuntimeError("dependency required-features need explicit resolution; refusing to omit a target")
            if missing:
                skipped.append((target_key(package["id"], target), "missing required-features: " + ", ".join(sorted(missing))))
                continue
            key = target_key(package["id"], target)
            groups.setdefault(selector, {})[key] = options.get("harness", True)
    if not groups:
        raise RuntimeError("Cargo metadata selected no test targets")
    return sorted(groups.items()), skipped


def command(selector):
    return ["cargo", "test", "--workspace", "--all-features", "--locked",
            "--no-fail-fast", "--message-format=json", *selector]


def remove_executables(target_dir, executables):
    """Remove emitted executables and hard links, never guessed hash prefixes.

    Cargo can hard-link an example's unhashed and hashed paths. Unlinking only
    one retains its disk blocks. Dependencies and unrelated files stay cached.
    """
    root = target_dir.resolve(strict=True)
    identities = set()
    for executable in executables:
        path = Path(executable)
        resolved = path.resolve(strict=True)
        if not resolved.is_relative_to(root) or path.is_symlink():
            raise RuntimeError(f"unsafe Cargo executable path: {path}")
        info = path.stat()
        if not stat.S_ISREG(info.st_mode) or not info.st_mode & 0o111:
            raise RuntimeError(f"not a regular executable: {path}")
        identities.add((info.st_dev, info.st_ino))
    for directory, _, filenames in os.walk(root, followlinks=False):
        for name in filenames:
            path = Path(directory, name)
            info = path.lstat()
            if stat.S_ISREG(info.st_mode) and (info.st_dev, info.st_ino) in identities:
                path.unlink()


class DiskGuard:
    def __init__(self, target_dir):
        self.target_dir = target_dir
        self.peak = 0
        self.minimum_free = None

    def sample(self):
        size = 0
        seen = set()
        for directory, _, filenames in os.walk(self.target_dir, followlinks=False):
            for name in filenames:
                try:
                    info = Path(directory, name).lstat()
                except FileNotFoundError:  # Cargo may replace its temporary output.
                    continue
                identity = (info.st_dev, info.st_ino)
                if stat.S_ISREG(info.st_mode) and identity not in seen:
                    size += info.st_size
                    seen.add(identity)
        free = shutil.disk_usage(self.target_dir).free
        self.peak = max(self.peak, size)
        self.minimum_free = free if self.minimum_free is None else min(self.minimum_free, free)
        if size > TARGET_LIMIT or free < FREE_MINIMUM:
            raise RuntimeError(f"disk guard: target={size} (limit {TARGET_LIMIT}), free={free} (minimum {FREE_MINIMUM})")


class SourceGuard:
    def __init__(self, root):
        self.root = root
        self.files = subprocess.check_output(["git", "ls-files", "-z"], cwd=root).split(b"\0")[:-1]
        self.records = {name: self.record(name) for name in self.files}

    def record(self, name):
        path = self.root / os.fsdecode(name)
        info = path.lstat()
        signature = (info.st_mode, info.st_size, info.st_mtime_ns, info.st_ctime_ns)
        content = os.readlink(path).encode() if path.is_symlink() else path.read_bytes()
        return signature, hashlib.sha256(content).digest()

    def check(self):
        files = subprocess.check_output(["git", "ls-files", "-z"], cwd=self.root).split(b"\0")[:-1]
        if files != self.files:
            raise RuntimeError("tracked source file inventory changed during validation")
        for name, (old_stat, digest) in self.records.items():
            path = self.root / os.fsdecode(name)
            info = path.lstat()
            current_stat = (info.st_mode, info.st_size, info.st_mtime_ns, info.st_ctime_ns)
            if current_stat != old_stat:
                current = self.record(name)
                if current[1] != digest or current_stat[0] != old_stat[0]:
                    raise RuntimeError(f"tracked source changed during validation: {os.fsdecode(name)}")
                self.records[name] = current


def classify(returncode, summaries, expected_harnesses, build_success, actual, expected):
    if not build_success or actual != expected:
        raise RuntimeError(f"incomplete compilation: expected {len(expected)} test artifacts, got {len(actual)}, exit {returncode}")
    if len(summaries) != expected_harnesses:
        raise RuntimeError(f"aborted or incomplete test run: expected {expected_harnesses} libtest summaries, got {len(summaries)}, exit {returncode}")
    failed = sum(int(match[2]) for match in summaries)
    if any((match[0] == "ok") != (int(match[2]) == 0) or int(match[5]) != 0 for match in summaries):
        raise RuntimeError("inconsistent test summary or unexpected filtered tests")
    if returncode == 0 and not failed:
        return False
    if returncode == 101 and failed:
        return True  # Ordinary assertions fail CI, but do not conceal later suites.
    raise RuntimeError(f"test process failed without a complete assertion-failure result: exit {returncode}")


def kill_group(process):
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def run_batch(selector, expected, seed, env, guard):
    env = dict(env, PROPTEST_RNG_SEED=seed)
    print(f"\n=== seed={seed} {' '.join(selector)} ===", flush=True)
    guard.sample()
    process = subprocess.Popen(command(selector), stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                               text=True, env=env, start_new_session=True)
    done = threading.Event()
    guard_errors = []

    def watch():
        while not done.wait(1):
            try:
                guard.sample()
                if process.poll() is not None:
                    # A finished Cargo can leave descendants holding stdout open.
                    kill_group(process)
                    return
            except Exception as error:
                guard_errors.append(error)
                kill_group(process)
                return

    monitor = threading.Thread(target=watch)
    monitor.start()
    artifacts, actual, summaries = set(), set(), []
    build_success = False
    compiling = True
    running = None
    summaries_by_executable = {}
    aborted = False
    try:
        for line in process.stdout:
            if not compiling:
                # Cargo owns executable boundaries. A failed test can include a
                # child's libtest summary in its captured diagnostics: retain the
                # final summary for each actual Cargo-run executable, not every
                # line resembling one across the whole batch.
                print(line, end="", flush=True)
                if line.lstrip().startswith("Running "):
                    for executable in artifacts:
                        if line.rstrip().endswith(f"({executable})"):
                            if executable in summaries_by_executable:
                                raise RuntimeError(f"Cargo ran a target twice: {executable}")
                            running = executable
                            summaries_by_executable[running] = None
                            break
                match = SUMMARY.match(line.rstrip())
                if match and running is not None:
                    summaries_by_executable[running] = match.groups()
                if "process didn't exit successfully:" in line and "signal:" in line:
                    aborted = True
                continue
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                print(line, end="", flush=True)
                continue
            if not isinstance(event, dict):
                print(line, end="", flush=True)
                continue
            if event.get("reason") == "compiler-artifact":
                key = target_key(event["package_id"], event["target"])
                if event["profile"]["test"] and event.get("executable"):
                    actual.add(key)
                    artifacts.add(event["executable"])
            elif event.get("reason") == "build-finished":
                build_success = event["success"]
                compiling = False
            elif event.get("reason") == "compiler-message":
                print(event["message"].get("rendered", ""), end="", flush=True)
            else:
                print(line, end="", flush=True)
        returncode = process.wait()
    finally:
        try:
            # Kill surviving descendants even if the Cargo leader already exited.
            kill_group(process)
            process.wait()
        finally:
            done.set()
            monitor.join()
            process.stdout.close()
    if guard_errors:
        raise guard_errors[0]
    guard.sample()
    if aborted or set(summaries_by_executable) != artifacts:
        raise RuntimeError("aborted or incomplete test run: Cargo did not complete every emitted executable")
    summaries = [summary for summary in summaries_by_executable.values() if summary is not None]
    failed = classify(returncode, summaries, sum(expected.values()), build_success, actual, set(expected))
    print(f"RESULT seed={seed} selector={list(selector)} exit={returncode} summaries={summaries}", flush=True)
    return failed, artifacts


def main():
    root = Path.cwd()
    source_guard = SourceGuard(root)
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--no-deps", "--all-features", "--locked", "--format-version=1"
    ]))
    plan, skipped = batches(metadata)
    for target, reason in skipped:
        print(f"EXCLUDED {target}: {reason}")
    print(f"PLAN {len(plan)} selectors, {sum(len(targets) for _, targets in plan)} test targets, seeds={SEEDS}", flush=True)
    # Do not change the OS stack limit or RUST_MIN_STACK. Core files from a
    # default-stack abort must not fill the runner while we report the failure.
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    failed = []
    with tempfile.TemporaryDirectory(prefix="jarde-tests-", dir=os.environ.get("RUNNER_TEMP")) as temporary:
        target_dir = Path(temporary)
        guard = DiskGuard(target_dir)
        env = dict(os.environ, CARGO_TARGET_DIR=str(target_dir), CARGO_BUILD_JOBS="1",
                   CARGO_INCREMENTAL="0", RUST_TEST_THREADS="1", CARGO_TERM_COLOR="never",
                   CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0")
        try:
            for selector, expected in plan:
                artifacts = set()
                for seed in SEEDS:
                    bad, emitted = run_batch(selector, expected, seed, env, guard)
                    artifacts.update(emitted)
                    if bad:
                        failed.append((selector, seed))
                    source_guard.check()
                remove_executables(target_dir, artifacts)
                guard.sample()
        finally:
            print(f"RESOURCE peak_target_bytes={guard.peak} minimum_free_bytes={guard.minimum_free}", flush=True)
            print(f"FAILED ASSERTION BATCHES {failed}", flush=True)
            source_guard.check()
    return bool(failed)


def interrupted(signum, _frame):
    raise InterruptedError(f"terminated by signal {signum}")


if __name__ == "__main__":
    signal.signal(signal.SIGTERM, interrupted)
    try:
        sys.exit(main())
    except (RuntimeError, OSError, subprocess.CalledProcessError) as error:
        print(f"::error::{error}", file=sys.stderr)
        sys.exit(1)
