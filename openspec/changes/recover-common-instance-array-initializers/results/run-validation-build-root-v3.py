from pathlib import Path
import datetime, hashlib, json, os, shutil, signal, subprocess, time
ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent / "validation-build-root-v3"
OUT.mkdir(exist_ok=False)
env = os.environ.copy()
env.update(CARGO_BUILD_JOBS="2", CARGO_INCREMENTAL="0", CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0")
commands = [['cargo', 'clippy', '-p', 'jarde', '-p', 'jarde-java', '--lib', '--tests', '--all-features', '--locked', '--', '-A', 'clippy::too_many_arguments', '-A', 'clippy::cloned_ref_to_slice_refs', '-A', 'clippy::collapsible_if', '-A', 'clippy::type_complexity', '-A', 'clippy::len_zero', '-A', 'clippy::needless_option_as_deref', '-A', 'clippy::needless_borrow', '-A', 'clippy::useless_conversion', '-A', 'clippy::large_enum_variant', '-A', 'clippy::question_mark', '-A', 'clippy::comparison_to_empty', '-A', 'clippy::op_ref', '-A', 'clippy::manual_range_patterns', '-A', 'clippy::if_same_then_else', '-A', 'clippy::filter_map_bool_then', '-A', 'clippy::filter_next', '-A', 'clippy::unneeded_struct_pattern', '-A', 'clippy::redundant_guards', '-A', 'clippy::map_identity', '-A', 'clippy::redundant_slicing', '-A', 'clippy::unnecessary_get_then_check', '-A', 'clippy::unnecessary_unwrap', '-A', 'clippy::redundant_locals', '-A', 'clippy::replace_box', '-A', 'clippy::map_clone', '-A', 'clippy::unnecessary_mut_passed', '-A', 'clippy::single_element_loop', '-A', 'clippy::unnecessary_to_owned', '-A', 'clippy::needless_lifetimes', '-D', 'warnings'], ['cargo', 'test', '-p', 'jarde', '--test', 'class_static_initializer_projection', '--test', 'interface_initializer_proof'], ['cargo', 'build', '-p', 'jarde-cli']]
records = []
for number, argv in enumerate(commands):
    if shutil.disk_usage(ROOT).free < 20 * 1024**3:
        raise SystemExit("20 GiB preflight disk guard")
    start = datetime.datetime.now(datetime.timezone.utc).isoformat()
    began = time.monotonic()
    out = OUT / f"{number}.stdout.raw"
    err = OUT / f"{number}.stderr.raw"
    reason = None
    peak = 0
    with out.open("wb") as stdout, err.open("wb") as stderr:
        child = subprocess.Popen(argv, cwd=ROOT, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
        while child.poll() is None:
            target = ROOT / "target"
            size = sum(p.stat().st_size for p in target.rglob("*") if p.is_file()) if target.exists() else 0
            peak = max(peak, size)
            free = shutil.disk_usage(ROOT).free
            if size > 1024**3 or free < 20 * 1024**3:
                reason = {"target_bytes": size, "free_bytes": free}
                os.killpg(child.pid, signal.SIGTERM)
                try:
                    child.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(child.pid, signal.SIGKILL)
                break
            time.sleep(1)
        exit_code = child.wait()
    records.append({"argv": argv, "cwd": str(ROOT), "started_at": start, "duration_seconds": time.monotonic()-began, "exit_code": exit_code, "guard_stop": reason, "peak_target_bytes": peak, "env_overrides": {key:env[key] for key in ["CARGO_BUILD_JOBS","CARGO_INCREMENTAL","CARGO_PROFILE_DEV_DEBUG","CARGO_PROFILE_TEST_DEBUG"]}, "streams": {key: {"path": str(path.relative_to(ROOT)), "bytes": path.stat().st_size, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for key,path in [("stdout",out),("stderr",err)]}})
    (OUT / "execution.json").write_text(json.dumps({"schema":"instance-array-validation-build-root-v3", "commands": records},indent=2)+"\n")
    print(json.dumps(records[-1]), flush=True)
    if exit_code:
        raise SystemExit(exit_code)
