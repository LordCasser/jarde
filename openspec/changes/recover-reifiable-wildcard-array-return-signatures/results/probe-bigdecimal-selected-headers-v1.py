"""Diagnostic renders only: exact original Main plus selected physical JDK headers.

This does not compile or execute targets and does not claim whole-source success.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent / "bigdecimal-selected-headers-v1"
CLI = Path("/private/tmp/jarde-wildcard-array-return-cli-v1")
RT = Path("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/jre/lib/rt.jar")
CLI_HASH = "196bb3e1e1bd074bb851f363938951a6d2dea81d8e895cd67e1a2560b3c2f761"
RT_HASH = "b27515a608ee447566b688e2bbb2257b1f0d8eceb96c307b87eb28d90a6630f4"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    assert sha(CLI.read_bytes()) == CLI_HASH
    assert sha(RT.read_bytes()) == RT_HASH
    OUT.mkdir(exist_ok=False)
    records = []
    with zipfile.ZipFile(RT) as runtime:
        headers = {name: runtime.read(name) for name in (
            "java/math/BigDecimal.class", "java/lang/Number.class")}
    for leg, ordinal in (("javac8", 20), ("javac23", 21)):
        original = OUT.parent / "candidate-root-v1" / "inputs" / f"{ordinal}-baseline-bigdecimal-control-corrected-{leg}.jar"
        with zipfile.ZipFile(original) as archive:
            entries = {name: archive.read(name) for name in archive.namelist()}
        for variant, selected in (
            ("original-only", ()),
            ("BigDecimal-only", ("java/math/BigDecimal.class",)),
            ("BigDecimal-and-Number", tuple(headers)),
        ):
            prefix = f"{leg}-{variant}"
            jar = OUT / f"{prefix}.jar"
            with zipfile.ZipFile(jar, "w", compression=zipfile.ZIP_DEFLATED) as archive:
                for name, data in {**entries, **{name: headers[name] for name in selected}}.items():
                    info = zipfile.ZipInfo(name, date_time=(2026, 10, 10, 0, 0, 0))
                    info.compress_type = zipfile.ZIP_DEFLATED
                    archive.writestr(info, data)
            argv = [str(CLI), "class-source", "--input", str(jar), "--class", "Main", "--policy", "plain-jar", "--format", "json", "--evidence", "all", "--release", "8"]
            result = subprocess.run(argv, cwd=ROOT, capture_output=True, timeout=45)
            stdout, stderr = OUT / f"{prefix}.stdout", OUT / f"{prefix}.stderr"
            stdout.write_bytes(result.stdout)
            stderr.write_bytes(result.stderr)
            report = json.loads(result.stdout)
            method = next(m for m in report["methods"] if m["item"]["name"]["escaped"] == "main")
            body = method["outcome"]["report"]
            records.append({
                "leg": leg, "variant": variant, "argv": argv, "cwd": str(ROOT), "exit": result.returncode,
                "original_jar_sha256": sha(original.read_bytes()), "jar": jar.name, "jar_sha256": sha(jar.read_bytes()),
                "main_class_sha256": sha(entries["Main.class"]),
                "selected_headers": {name: sha(headers[name]) for name in selected},
                "stdout": stdout.name, "stdout_sha256": sha(result.stdout), "stderr": stderr.name, "stderr_sha256": sha(result.stderr),
                "execution": body["execution"], "quality": body["quality"], "representation": body["representation"],
                "main_text": method["text"], "diagnostics": body["diagnostics"], "news": body["news"], "concats": body["concats"],
                "runtime_coverage": report["coverage"]["runtime_resolution"],
            })
    manifest = {"purpose": "diagnostic render with actual selected snapshot headers; no compile/runtime/whole-program acceptance", "cli_sha256": CLI_HASH, "rt_jar": str(RT), "rt_sha256": RT_HASH, "records": records}
    (OUT / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    for record in records:
        print(record["leg"], record["variant"], record["exit"], record["quality"], [d["code"] for d in record["diagnostics"]])


if __name__ == "__main__":
    main()
