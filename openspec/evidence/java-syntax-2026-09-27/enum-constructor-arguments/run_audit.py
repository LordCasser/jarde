#!/usr/bin/env python3
"""Rebuild the frozen DT-11 enum constructor-argument comparison."""
from __future__ import annotations
import hashlib, json, os, shutil, subprocess, tempfile, zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path(os.environ.get("JADX_CHECKOUT", "/Users/lordcasser/workspace/testzone/jadx"))
JADX = Path(os.environ.get("JADX", str(JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx")))
JARDE_CLI = os.environ.get("JARDE_CLI")
GENERATED = HERE / "generated"
TEMP_PREFIXES: list[tuple[str, str]] = []
SOURCES = ["LiteralOnly.java", "IntArgs.java", "Ints.java", "StringVarargs.java",
           "LiteralOnlyRunner.java", "IntArgsRunner.java", "StringVarargsRunner.java"]
ENUMS = ["LiteralOnly", "IntArgs", "StringVarargs"]
EXPECTED_OUTPUTS = {name: f"OK {name}\n" for name in ENUMS}
EXPECTED_ENUM_SHA256 = {
    "LiteralOnly": "c3e9d60bf01f9aef37ff373d66bf47743dc8584b69b747008b5dbf3780646b16",
    "IntArgs": "2b8047d88946b349053dee0785e759f806308b4a78fda9fdc23e79337373cef0",
    "StringVarargs": "a6cb2a1be5580c16429560fa88b9eb3d32a8caa5927d1975d9fee28f730bbac7",
}
JAVAC = ["javac", "-J-Duser.language=en", "-J-Duser.country=US"]

# Only this script's derived logs and decompiler sources are replaced. All javac class output,
# JADX working directories, and the Cargo target live in TemporaryDirectory and are removed.
if GENERATED.exists():
    shutil.rmtree(GENERATED)
GENERATED.mkdir()

def sha(path: Path) -> str:
    h=hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda:f.read(1024*1024),b""): h.update(chunk)
    return h.hexdigest()

def clean_log(text: str) -> str:
    return "\n".join(line.rstrip() for line in text.splitlines()) + "\n"

def save_run(name: str, args: list[str], *, cwd: Path = ROOT, env=None, combine=False,
             record=True, normalize=()):
    p = subprocess.run(args, cwd=cwd, env=env, text=True,
                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT if combine else subprocess.PIPE)
    out = p.stdout or ""
    err = "" if combine else (p.stderr or "")
    if record:
        text = out + err if combine else out
        for old,new in [*normalize, *TEMP_PREFIXES]:
            text=text.replace(old,new)
        (GENERATED / f"{name}.log").write_text(clean_log(text))
        if not combine:
            detail=err
            for old,new in [*normalize, *TEMP_PREFIXES]:
                detail=detail.replace(old,new)
            (GENERATED / f"{name}.stderr.log").write_text(clean_log(detail))
    return p, out, err

def checked(label, args, **kwargs):
    p, out, err = save_run(label, list(map(str,args)), **kwargs)
    if p.returncode != 0:
        raise SystemExit(f"{label} failed ({p.returncode}); inspect the input or tool configuration")
    return p, out, err

versions=[]
for name,args in [("javac-version",["javac","-version"]),("java-version",["java","-version"]),("javap-version",["javap","-version"]),("jadx-version",[str(JADX),"--version"])]:
    checked(name,args,combine=True)
versions.append(f"JADX path: {JADX}")
versions.append(f"JADX checkout: {JADX_ROOT}")
if JADX_ROOT.exists():
    p=subprocess.run(["git","-C",str(JADX_ROOT),"rev-parse","HEAD"],text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=True)
    jadx_commit=p.stdout.strip()
    versions.append(f"JADX checkout commit: {jadx_commit}")
    if jadx_commit != "2fb1b16386941660fda07e9017285aec40fcb37f":
        raise SystemExit("DT-11 evidence requires JADX checkout 2fb1b16386941660fda07e9017285aec40fcb37f")
else:
    raise SystemExit("Set JADX_CHECKOUT to the fixed JADX source checkout")
versions.append(f"JARDE CLI source: {'provided binary' if JARDE_CLI else 'cargo run -p jarde-cli from repository checkout'}")

with tempfile.TemporaryDirectory(prefix="dt11-audit-") as scratch:
    scratch=Path(scratch)
    TEMP_PREFIXES.append((str(scratch), "<TEMP>"))
    env=os.environ.copy(); env["CARGO_TARGET_DIR"]=str(scratch/"cargo-target"); env["CARGO_TERM_COLOR"]="never"
    original_classes=scratch/"original-classes"; original_classes.mkdir()
    statuses={"original":{},"jadx":{},"jarde":{}}
    p,_,_=checked("original-javac",[*JAVAC,"--release","8","-g","-d",str(original_classes),*map(str,[HERE/s for s in SOURCES])],combine=True)
    statuses["original"]["javac"]=p.returncode
    for enum, expected_hash in EXPECTED_ENUM_SHA256.items():
        if sha(original_classes/f"{enum}.class") != expected_hash:
            raise SystemExit(f"frozen original class changed: {enum}")
    for enum in ENUMS:
        runner=enum+"Runner"
        p,out,_=checked(f"original-run-{enum}",["java","-Xverify:all","-cp",str(original_classes),runner],combine=True)
        if out != EXPECTED_OUTPUTS[enum]: raise SystemExit(f"original runtime output changed: {enum}")
        statuses["original"][enum+"_run"]=p.returncode
    classpath_jar=scratch/"original-classpath.jar"
    with zipfile.ZipFile(classpath_jar,"w",zipfile.ZIP_DEFLATED) as archive:
        for class_file in sorted(original_classes.glob("*.class")):
            archive.write(class_file, class_file.name)
    for enum in ENUMS+["Ints"]:
        p,_,_=checked(f"javap-{enum}",["javap","-v","-p","-c","-classpath",str(original_classes),enum],combine=True,record=False)
        # Remove the volatile filesystem mtime and scratch path; retain class hash, full structure and Code.
        text=p.stdout.replace(str(original_classes),"<TEMP>/original-classes")
        text="\n".join(line for line in text.splitlines() if not line.startswith("  Last modified "))+"\n"
        (GENERATED/f"javap-{enum}.log").write_text(text)
    for enum in ENUMS:
        runner=enum+"Runner"
        for tool in ("jadx","jarde"):
            if tool=="jadx":
                jadx_dir=scratch/f"jadx-{enum}"
                p,_,_=checked(f"jadx-{enum}",[str(JADX),"-d",str(jadx_dir),str(original_classes/f"{enum}.class")],combine=True)
                found=list(jadx_dir.rglob(enum+".java"))
                if len(found)!=1: raise SystemExit(f"JADX emitted {len(found)} files named {enum}.java")
                source=GENERATED/"jadx"/f"{enum}.java"; source.parent.mkdir(exist_ok=True)
                shutil.copyfile(found[0],source)
            else:
                if enum=="IntArgs":
                    jarde_input=classpath_jar
                    policy="plain-jar"
                else:
                    jarde_input=original_classes/f"{enum}.class"
                    policy="single-class"
                cmd=["cargo","run","--quiet","-p","jarde-cli","--","class-source","--input",str(jarde_input),"--class",enum,"--policy",policy,"--format","text"] if not JARDE_CLI else [JARDE_CLI,"class-source","--input",str(jarde_input),"--class",enum,"--policy",policy,"--format","text"]
                p,out,_=save_run(f"jarde-{enum}",cmd,cwd=ROOT,env=env,record=False)
                if p.returncode != 0: raise SystemExit(f"Jarde class-source failed ({p.returncode})")
                source=GENERATED/f"jarde-{enum}.java"; source.write_text(out)
            # Compilation uses a scratch source set: JADX's defpackage spelling needs its runner/helper
            # in that package, while Jarde and javac fixture classes use the default package.
            case=scratch/f"compile-{tool}-{enum}"; case.mkdir()
            if tool=="jadx":
                compile_source=case/f"{enum}.java"
                shutil.copyfile(source,compile_source)
                runner_source="package defpackage;\n"+(HERE/f"{runner}.java").read_text()
                (case/f"{runner}.java").write_text(runner_source)
                if enum=="IntArgs":
                    (case/"Ints.java").write_text("package defpackage;\n"+(HERE/"Ints.java").read_text())
                runtime_name=f"defpackage.{runner}"
            else:
                compile_source=case/f"{enum}.java"
                shutil.copyfile(source,compile_source)
                shutil.copyfile(HERE/f"{runner}.java",case/f"{runner}.java")
                if enum=="IntArgs": shutil.copyfile(HERE/"Ints.java",case/"Ints.java")
                runtime_name=runner
            inputs=[compile_source,case/f"{runner}.java"]
            if enum=="IntArgs": inputs.append(case/"Ints.java")
            class_out=case/"classes"; class_out.mkdir()
            cp,compile_output,_=save_run(f"{tool}-javac-{enum}",[*JAVAC,"--release","8","-d",str(class_out),*map(str,inputs)],combine=True)
            statuses[tool][enum+"_javac"]=cp.returncode
            if cp.returncode==0:
                rp,runtime_output,_=save_run(f"{tool}-run-{enum}",["java","-Xverify:all","-cp",str(class_out),runtime_name],combine=True)
                if runtime_output != EXPECTED_OUTPUTS[enum]:
                    raise SystemExit(f"{tool} runtime output differs from original: {enum}")
                statuses[tool][enum+"_run"]=rp.returncode
            else:
                raise SystemExit(f"{tool} source failed Java 8 compilation: {enum}; inspect generated/{tool}-javac-{enum}.log")
    versions.append(f"JADX launcher SHA-256: {sha(JADX)}")
    for rel in ["jadx-cli/build/install/jadx/lib/jadx-cli-dev.jar","jadx-cli/build/install/jadx/lib/jadx-core-dev.jar"]:
        p=JADX_ROOT/rel
        if p.exists(): versions.append(f"{rel} SHA-256: {sha(p)}")
    (GENERATED/"tool-versions.txt").write_text("\n".join(versions)+"\n")
    hashes=[]
    for p in sorted(HERE.glob("*.java")):
        hashes.append(f"{sha(p)}  input/{p.name}")
    for p in sorted((HERE/"reference").glob("*.java")):
        hashes.append(f"{sha(p)}  reference/{p.name}")
    for p in sorted(GENERATED.rglob("*.java")):
        hashes.append(f"{sha(p)}  output/{p.relative_to(GENERATED)}")
    for p in sorted(original_classes.glob("*.class")):
        hashes.append(f"{sha(p)}  Java-8-class/{p.name}")
    for p in sorted(JADX_ROOT.glob("jadx-cli/build/install/jadx/lib/jadx-*-dev.jar")):
        hashes.append(f"{sha(p)}  tool/{p.name}")
    (GENERATED/"sha256.txt").write_text("\n".join(hashes)+"\n")
    expected={"LiteralOnly_javac":0,"LiteralOnly_run":0,"IntArgs_javac":0,"IntArgs_run":0,"StringVarargs_javac":0,"StringVarargs_run":0}
    if statuses["original"] != {"javac":0,"LiteralOnly_run":0,"IntArgs_run":0,"StringVarargs_run":0}:
        raise SystemExit("Original fixture compilation/runtime results changed")
    if statuses["jadx"] != expected:
        raise SystemExit("JADX compilation/runtime results changed")
    if statuses["jarde"] != expected:
        raise SystemExit("Jarde compilation/runtime results changed")
    (GENERATED/"summary.json").write_text(json.dumps({"status":statuses,"expected":{"original":"all pass","jadx":"all compile and run","jarde":"all compile and run"}},indent=2,ensure_ascii=False)+"\n")
print((GENERATED/"summary.json").read_text())
