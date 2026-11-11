import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import Mock, patch

import bounded_workspace_tests as runner


class Workspace(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.addCleanup(self.temporary.cleanup)

    def package(self, name, targets, extra="", features=None):
        manifest = self.root / (name + ".toml")
        manifest.write_text('[package]\nname = "' + name + '"\nversion = "0.1.0"\n' + extra)
        return dict(id=name, name=name, manifest_path=str(manifest), targets=targets, features=features or {})

    def target(self, kind, name, **changes):
        return dict(kind=[kind], name=name, test=True, **changes)

    def metadata(self, *packages):
        return dict(packages=list(packages), workspace_members=[p["id"] for p in packages])

    def test_all_target_kinds_and_dependencies(self):
        example = self.target("example", "example")
        example["test"] = False
        package = self.package("a", [self.target("lib", "a"), self.target("bin", "cli"),
            self.target("test", "integration"), self.target("bench", "bench"), example,
            self.target("custom-build", "build-script-build")])
        metadata = self.metadata(package)
        metadata["packages"].append(self.package("external", [self.target("lib", "external")]))
        plan, skipped = runner.batches(metadata)
        self.assertEqual({s for s, _ in plan}, {("--lib",), ("--bin", "cli"),
            ("--test", "integration"), ("--bench", "bench"), ("--example", "example")})
        self.assertEqual(sum(map(len, (targets for _, targets in plan))), 5)
        self.assertEqual(skipped, [])

    def test_workspace_shared_names_keep_every_package(self):
        packages = [self.package(name, [self.target("lib", name), self.target("test", "shared")])
                    for name in ("a", "b")]
        plan, _ = runner.batches(self.metadata(*packages))
        self.assertEqual([len(targets) for _, targets in plan], [2, 2])
        for selector, _ in plan:
            command = runner.command(selector)
            for option in ("--workspace", "--all-features", "--locked", "--no-fail-fast"):
                self.assertIn(option, command)
            self.assertNotIn("-p", command)
            self.assertNotIn("--exclude", command)

    def test_disabled_targets_required_features_and_custom_harness(self):
        disabled = self.target("test", "disabled")
        disabled["test"] = False
        enabled_bench = self.target("test", "enabled_bench")
        enabled_bench["test"] = False
        needed = self.target("test", "needed", **{"required-features": ["available"]})
        absent = self.target("test", "absent", **{"required-features": ["missing"]})
        package = self.package("a", [disabled, enabled_bench, needed, absent],
            '\n[[test]]\nname = "disabled"\nbench = false\n'
            '\n[[test]]\nname = "enabled_bench"\nbench = true\nharness = false\n',
            features={"available": []})
        plan, skipped = runner.batches(self.metadata(package))
        self.assertEqual([s for s, _ in plan], [("--test", "enabled_bench"), ("--test", "needed")])
        self.assertEqual(list(plan[0][1].values()), [False])
        self.assertEqual(len(skipped), 2)
        self.assertTrue(any("missing required-features" in reason for _, reason in skipped))

    def test_library_is_not_gated_by_required_features(self):
        library = self.target("lib", "a", **{"required-features": ["absent"]})
        plan, skipped = runner.batches(self.metadata(self.package("a", [library])))
        self.assertEqual(len(plan), 1)
        self.assertEqual(skipped, [])

    def test_unknown_selection_never_silently_disappears(self):
        for target in (self.target("future-kind", "a"), self.target("test", "a", **{"required-features": ["dep/feature"]})):
            with self.subTest(target=target), self.assertRaises(RuntimeError):
                runner.batches(self.metadata(self.package("a", [target])))
        with self.assertRaises(RuntimeError):
            runner.batches(self.metadata())

    def executable(self, name="binary"):
        path = self.root / name
        path.write_text("executable")
        path.chmod(0o755)
        return path

    def test_cleanup_removes_only_reported_inode_and_its_aliases(self):
        executable = self.executable()
        alias = self.root / "unhashed-example"
        alias.hardlink_to(executable)
        unrelated = self.executable("binary-similar-name")
        dependency = self.root / "dependency.rlib"
        dependency.write_bytes(b"cached")
        symlink = self.root / "symlink"
        symlink.symlink_to(executable)
        runner.remove_executables(self.root, [str(executable)])
        self.assertFalse(executable.exists())
        self.assertFalse(alias.exists())
        self.assertTrue(unrelated.exists())
        self.assertTrue(dependency.exists())
        self.assertTrue(symlink.is_symlink())

    def test_cleanup_rejects_outside_symlink_and_nonexecutable(self):
        nested = self.root / "target"
        nested.mkdir()
        outside = self.executable()
        symlink = nested / "escape"
        symlink.symlink_to(outside)
        nonexecutable = nested / "library.rlib"
        nonexecutable.write_text("library")
        for path in (outside, symlink, nonexecutable):
            with self.subTest(path=path), self.assertRaises(RuntimeError):
                runner.remove_executables(nested, [str(path)])
        self.assertTrue(outside.exists())
        self.assertTrue(nonexecutable.exists())

    def test_disk_guard_boundary_hardlink_dedup_and_low_space(self):
        path = self.root / "artifact"
        path.write_bytes(b"12345678")
        (self.root / "alias").hardlink_to(path)
        usage = Mock(free=10)
        with patch.object(runner, "TARGET_LIMIT", 8), patch.object(runner, "FREE_MINIMUM", 10), \
                patch.object(runner.shutil, "disk_usage", return_value=usage):
            guard = runner.DiskGuard(self.root)
            guard.sample()
            self.assertEqual(guard.peak, 8)
            path.write_bytes(b"123456789")
            with self.assertRaisesRegex(RuntimeError, "disk guard"):
                guard.sample()
            path.unlink()
            usage.free = 9
            with self.assertRaisesRegex(RuntimeError, "disk guard"):
                guard.sample()

    def test_source_guard_allows_touch_but_rejects_content_or_inventory_changes(self):
        source = self.root / "source.rs"
        source.write_text("before")
        with patch.object(runner.subprocess, "check_output", return_value=b"source.rs\0") as tracked:
            guard = runner.SourceGuard(self.root)
            os.utime(source, None)
            guard.check()
            source.write_text("after!")
            with self.assertRaisesRegex(RuntimeError, "tracked source changed"):
                guard.check()
            tracked.return_value = b"source.rs\0extra.rs\0"
            with self.assertRaisesRegex(RuntimeError, "inventory changed"):
                guard.check()

    def run_synthetic(self, body, guard=None, harness=True):
        target = self.target("test", "synthetic")
        executable = self.executable()
        event = dict(reason="compiler-artifact", package_id="a", target=target,
                     profile={"test": True}, executable=str(executable))
        script = self.root / "fake_cargo.py"
        script.write_text("import json, os, sys, time\n" +
            "print(" + repr(json.dumps(event)) + ", flush=True)\n" +
            "print('{\"reason\":\"build-finished\",\"success\":true}', flush=True)\n" +
            "print(" + repr(f"     Running tests/synthetic.rs ({executable})") + ", flush=True)\n" + body)
        expected = {runner.target_key("a", target): harness}
        self.output = io.StringIO()
        with patch.object(runner, "command", return_value=[sys.executable, "-u", str(script)]), \
                contextlib.redirect_stdout(self.output):
            return runner.run_batch(("--test", "synthetic"), expected, runner.SEEDS[0], os.environ,
                                    guard or Mock())

    def test_standard_libtest_preserves_should_panic_and_seed(self):
        bad, artifacts = self.run_synthetic(
            f"assert os.environ['PROPTEST_RNG_SEED'] == {runner.SEEDS[0]!r}\n"
            "print('test panics - should panic ... ok')\n"
            "print('42')\n"  # Non-event JSON from test output is not a Cargo event.
            "print('test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s')\n")
        self.assertFalse(bad)
        self.assertEqual(len(artifacts), 1)

    def test_custom_harness_preserves_exit_status_and_unknown_json_output(self):
        bad, artifacts = self.run_synthetic(
            "print('{\"custom_output\": \"visible\"}')\n", harness=False)
        self.assertFalse(bad)
        self.assertEqual(len(artifacts), 1)
        self.assertIn('"custom_output": "visible"', self.output.getvalue())
        with self.assertRaises(RuntimeError):
            self.run_synthetic("sys.exit(1)\n", harness=False)

    def test_assertion_failure_is_retained_and_can_continue(self):
        bad, _ = self.run_synthetic(
            "print('test failing ... FAILED')\n"
            "print('test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s')\n"
            "sys.exit(101)\n")
        self.assertTrue(bad)

    def test_captured_child_summary_uses_outer_result_and_continues(self):
        bad, _ = self.run_synthetic(
            "print('failures:\\n---- marker stdout ----')\n"
            "print('test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s')\n"
            "print('test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s')\n"
            "sys.exit(101)\n")
        self.assertTrue(bad)

    def test_runtime_json_cannot_mutate_compiler_evidence(self):
        bad, _ = self.run_synthetic(
            "print('{\"reason\":\"compiler-artifact\"}')\n"
            "print('{\"reason\":\"build-finished\",\"success\":false}')\n"
            "print('test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s')\n")
        self.assertFalse(bad)
        self.assertIn('"success":false', self.output.getvalue())

    def test_signal_report_after_nested_summary_is_fatal(self):
        with self.assertRaisesRegex(RuntimeError, "aborted"):
            self.run_synthetic(
                "print('test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s')\n"
                "print(\"  process didn't exit successfully: binary (signal: 6, SIGABRT)\")\n"
                "sys.exit(101)\n")

    def test_abort_missing_summary_or_list_only_is_not_success(self):
        for output in ("sys.exit(101)\n", "print('test_a: test\\n1 test, 0 benchmarks')\n", "os.kill(os.getpid(), 9)\n"):
            with self.subTest(output=output), self.assertRaisesRegex(RuntimeError, "incomplete test run"):
                self.run_synthetic(output)

    def test_orphan_with_inherited_stdout_cannot_hang_finished_cargo(self):
        started = time.monotonic()
        bad, _ = self.run_synthetic(
            "print('test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s')\n"
            "if os.fork() == 0:\n    time.sleep(20)\n"
            "sys.exit(0)\n")
        self.assertFalse(bad)
        self.assertLess(time.monotonic() - started, 5)

    def test_guard_kills_running_process_and_propagates(self):
        guard = Mock()
        guard.sample.side_effect = [None, RuntimeError("disk guard triggered")]
        with self.assertRaisesRegex(RuntimeError, "disk guard triggered"):
            self.run_synthetic("time.sleep(20)\n", guard)
        self.assertEqual(guard.sample.call_count, 2)

    def test_filtered_status_compilation_and_exit_mismatches_fail(self):
        good = [("ok", "1", "0", "0", "0", "0")]
        variants = [(-9, good, 1, True, {"a"}, {"a"}),
                    (0, good, 1, False, {"a"}, {"a"}),
                    (0, good, 1, True, set(), {"a"}),
                    (0, [("ok", "0", "0", "0", "0", "1")], 1, True, {"a"}, {"a"}),
                    (0, [("FAILED", "0", "1", "0", "0", "0")], 1, True, {"a"}, {"a"})]
        for args in variants:
            with self.subTest(args=args), self.assertRaises(RuntimeError):
                runner.classify(*args)

    def test_sigterm_unwinds_main_and_removes_private_target(self):
        ready = self.root / "ready"
        child = self.root / "interrupt.py"
        child.write_text(
            "import os, signal, sys, time\nfrom pathlib import Path\nfrom unittest.mock import patch\n"
            + f"sys.path.insert(0, {str(Path(runner.__file__).parent)!r})\n"
            + "import bounded_workspace_tests as b\n"
            + "def run(selector, expected, seed, env, guard):\n"
            + f"    Path({str(ready)!r}).write_text(env['CARGO_TARGET_DIR'])\n"
            + "    while True: time.sleep(1)\n"
            + "signal.signal(signal.SIGTERM, b.interrupted)\n"
            + "with patch.object(b, 'SourceGuard'), patch.object(b, 'DiskGuard'), "
            + "patch.object(b.subprocess, 'check_output', return_value=b'{}'), "
            + "patch.object(b, 'batches', return_value=([((' --test', 'a'), {})], [])), "
            + "patch.object(b, 'run_batch', side_effect=run):\n"
            + "    try: b.main()\n"
            + "    except InterruptedError: sys.exit(143)\n")
        process = subprocess.Popen([sys.executable, str(child)], stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, env=dict(os.environ, RUNNER_TEMP=str(self.root)))
        try:
            deadline = time.monotonic() + 5
            while not ready.exists() and process.poll() is None and time.monotonic() < deadline:
                time.sleep(0.01)
            self.assertTrue(ready.exists())
            private_target = Path(ready.read_text())
            self.assertTrue(private_target.exists())
            process.terminate()
            output, error = process.communicate(timeout=5)
            self.assertEqual(process.returncode, 143, (output, error))
            self.assertFalse(private_target.exists())
        finally:
            if process.poll() is None:
                process.kill()
            process.communicate()

    def test_main_continues_failures_runs_both_seeds_and_finally_cleans(self):
        for fatal in (False, True):
            with self.subTest(fatal=fatal):
                created = []
                calls = []
                def run(selector, expected, seed, env, guard):
                    directory = Path(env["CARGO_TARGET_DIR"])
                    created.append(directory)
                    calls.append((selector, seed))
                    executable = directory / "emitted"
                    executable.write_text("native code")
                    executable.chmod(0o755)
                    if fatal:
                        raise RuntimeError("aborted")
                    return True, {str(executable)}
                with patch.object(runner, "SourceGuard"), patch.object(runner, "DiskGuard"), \
                    patch.object(runner.subprocess, "check_output", return_value=b"{}"), \
                    patch.object(runner, "batches", return_value=([(("--test", "a"), {}), (("--test", "b"), {})], [])), \
                    patch.object(runner, "run_batch", side_effect=run), \
                    patch.object(runner.resource, "setrlimit"), \
                    patch.dict(os.environ, {"RUNNER_TEMP": str(self.root)}), contextlib.redirect_stdout(io.StringIO()):
                    if fatal:
                        with self.assertRaisesRegex(RuntimeError, "aborted"):
                            runner.main()
                    else:
                        self.assertTrue(runner.main())
                        self.assertEqual([seed for _, seed in calls], list(runner.SEEDS) * 2)
                self.assertTrue(created)
                self.assertTrue(all(not directory.exists() for directory in created))


if __name__ == "__main__":
    unittest.main()
