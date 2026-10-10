from pathlib import Path
import datetime, hashlib, json, os, shutil, signal, subprocess, time
ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent / "scoped-rust-root-v1"
OUT.mkdir(exist_ok=False)
env = os.environ.copy()
env.update(CARGO_BUILD_JOBS="2", CARGO_INCREMENTAL="0", CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0")
commands = [["cargo", "test", "-p", "jarde-java", "--lib", "instance_array", "--", "--nocapture"], ["cargo", "test", "-p", "jarde", "--lib", "common_instance_array_initializer_tests", "--", "--nocapture"]]
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
    (OUT / "execution.json").write_text(json.dumps({"schema":"instance-array-scoped-rust-root-v1", "commands": records},indent=2)+"\n")
    print(json.dumps(records[-1]), flush=True)
    if exit_code:
        raise SystemExit(exit_code)
