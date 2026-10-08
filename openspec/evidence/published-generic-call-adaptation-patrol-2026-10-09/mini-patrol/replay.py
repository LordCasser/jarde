#!/usr/bin/env python3
"""Read-only source/bytecode adaptation probes; outputs stay under this /tmp directory."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

OUT = Path(__file__).resolve().parent
JDK = Path("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home")
JAR = JDK / "bin/jar"
JAVAC = JDK / "bin/javac"
JAVA = JDK / "bin/java"
JAVAP = JDK / "bin/javap"
JARDE = Path("/tmp/jarde-raw-receiver-final-v3-cli")
JADX = Path("/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx")

SOURCES = {
    "EmptySink": """public class EmptySink<T> {
    public void sink(T x) {}
}
""",
    "FieldSetter": """public class FieldSetter<T> {
    public T value;
    public void set(T x) { value = x; }
}
""",
    "CallRelay": """public class CallRelay<T> {
    public T identity(T x) { return x; }
    public T relay(T x) { return identity(x); }
}
""",
    "TypedSetter": """public class TypedSetter<T> {
    public T value;
    public void set(TypedSetter<T> c, T x) { c.value = x; }
}
""",
}

DRIVER = r'''import java.lang.reflect.Field;
import java.lang.reflect.Method;

public class ProbeDriver {
    private static void describe(String label, Method method) {
        System.out.println(label + ".genericParameters=" + java.util.Arrays.toString(method.getGenericParameterTypes()));
        System.out.println(label + ".genericReturn=" + method.getGenericReturnType());
        System.out.println(label + ".erasedParameters=" + java.util.Arrays.toString(method.getParameterTypes()));
        System.out.println(label + ".erasedReturn=" + method.getReturnType().getTypeName());
    }
    public static void main(String[] args) throws Exception {
        String probe = args[0];
        Class<?> type = Class.forName(args[1]);
        Object marker = new Object();
        if (probe.equals("EmptySink")) {
            describe("sink", type.getMethod("sink", Object.class));
        } else if (probe.equals("FieldSetter")) {
            Field field = type.getField("value");
            Method set = type.getMethod("set", Object.class);
            System.out.println("field.generic=" + field.getGenericType());
            describe("set", set);
            Object instance = type.getConstructor().newInstance();
            set.invoke(instance, marker);
            System.out.println("behavior.field-is-marker=" + (field.get(instance) == marker));
        } else if (probe.equals("CallRelay")) {
            describe("identity", type.getMethod("identity", Object.class));
            Method relay = type.getMethod("relay", Object.class);
            describe("relay", relay);
            Object instance = type.getConstructor().newInstance();
            System.out.println("behavior.relay-is-marker=" + (relay.invoke(instance, marker) == marker));
        } else if (probe.equals("TypedSetter")) {
            Field field = type.getField("value");
            Method set = type.getMethod("set", type, Object.class);
            System.out.println("field.generic=" + field.getGenericType());
            describe("set", set);
            Object instance = type.getConstructor().newInstance();
            set.invoke(instance, instance, marker);
            System.out.println("behavior.field-is-marker=" + (field.get(instance) == marker));
        } else {
            throw new IllegalArgumentException(probe);
        }
    }
}
'''


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(label, argv, cwd, save_dir, commands):
    argv = [str(x) for x in argv]
    result = subprocess.run(argv, cwd=cwd, capture_output=True)
    (save_dir / (label + ".stdout")).write_bytes(result.stdout)
    (save_dir / (label + ".stderr")).write_bytes(result.stderr)
    commands.append({
        "label": label, "argv": argv, "cwd": str(cwd), "returncode": result.returncode,
        "stdout_sha256": sha(save_dir / (label + ".stdout")),
        "stderr_sha256": sha(save_dir / (label + ".stderr")),
    })
    return result


def main():
    commands = []
    cases = []
    for name, source in SOURCES.items():
        area = OUT / name
        area.mkdir()
        original_dir = area / "original"
        original_dir.mkdir()
        source_path = original_dir / (name + ".java")
        source_path.write_text(source)
        driver_path = area / "ProbeDriver.java"
        driver_path.write_text(DRIVER)

        jar_path = area / (name + ".jar")
        with tempfile.TemporaryDirectory(prefix="next-call-input-") as temp:
            temp = Path(temp)
            classes = temp / "classes"
            empty_cp = temp / "empty-classpath"
            empty_sp = temp / "empty-sourcepath"
            classes.mkdir(); empty_cp.mkdir(); empty_sp.mkdir()
            compiled = run("freeze-javac", [JAVAC, "-source", "8", "-target", "8", "-g",
                          "-classpath", empty_cp, "-sourcepath", empty_sp, "-d", classes,
                          source_path], area, original_dir, commands)
            if compiled.returncode == 0:
                class_sha = sha(classes / (name + ".class"))
                run("freeze-jar", [JAR, "cf", jar_path, "-C", classes, name + ".class"],
                    area, original_dir, commands)
            else:
                class_sha = None

        entry = {
            "probe": name,
            "source_sha256": sha(source_path),
            "input_class_sha256": class_sha,
            "input_jar_sha256": sha(jar_path) if jar_path.exists() else None,
            "flavors": {},
        }
        if class_sha is None:
            cases.append(entry)
            continue

        jarde_dir = area / "jarde"
        jarde_dir.mkdir()
        emitted = run("class-source", [JARDE, "class-source", "--input", jar_path, "--class", name,
                      "--policy", "plain-jar", "--release", "8", "--format", "text"],
                      area, jarde_dir, commands)
        (jarde_dir / (name + ".java")).write_bytes(emitted.stdout)

        jadx_dir = area / "jadx"
        jadx_dir.mkdir()
        full_output = jadx_dir / "full-output"
        full_output.mkdir()
        decompiled = run("jadx", [JADX, "--no-res", "-d", full_output, jar_path],
                         area, jadx_dir, commands)
        jadx_sources = list(full_output.rglob(name + ".java"))
        jadx_source = jadx_sources[0] if len(jadx_sources) == 1 else None

        flavor_sources = {
            "original": (source_path, name, original_dir),
            "jarde": (jarde_dir / (name + ".java"), name, jarde_dir),
            "jadx": (jadx_source, None, jadx_dir),
        }
        for flavor, (flavor_source, class_name, flavor_dir) in flavor_sources.items():
            row = {"source_sha256": sha(flavor_source) if flavor_source else None}
            if flavor_source is None:
                row["compile_exit"] = None
                row["note"] = f"expected one full-class source, saw {len(jadx_sources)}"
                entry["flavors"][flavor] = row
                continue
            text = flavor_source.read_text(errors="replace")
            package = None
            for line in text.splitlines():
                if line.strip().startswith("package ") and line.strip().endswith(";"):
                    package = line.strip()[8:-1]
                    break
            if flavor == "jadx":
                class_name = (package + "." if package else "") + name
            with tempfile.TemporaryDirectory(prefix=f"next-call-{flavor}-") as temp:
                temp = Path(temp)
                classes = temp / "classes"
                empty_cp = temp / "empty-classpath"
                empty_sp = temp / "empty-sourcepath"
                classes.mkdir(); empty_cp.mkdir(); empty_sp.mkdir()
                compiled = run(flavor + "-javac", [JAVAC, "-source", "8", "-target", "8", "-g",
                              "-classpath", empty_cp, "-sourcepath", empty_sp, "-d", classes,
                              flavor_source, driver_path], area, flavor_dir, commands)
                row["compile_exit"] = compiled.returncode
                row["compiled_class_hashes"] = {
                    str(path.relative_to(classes)): sha(path)
                    for path in sorted(classes.rglob("*.class"))
                }
                if compiled.returncode == 0:
                    executed = run(flavor + "-java", [JAVA, "-Xverify:all", "-cp", classes,
                                  "ProbeDriver", name, class_name], area, flavor_dir, commands)
                    row["runtime_exit"] = executed.returncode
                    row["runtime_stdout"] = executed.stdout.decode(errors="replace")
                    row["runtime_stderr"] = executed.stderr.decode(errors="replace")
            entry["flavors"][flavor] = row
        cases.append(entry)

    manifest = {
        "scope": "Exploratory four-probe mini-patrol; outputs confined to /tmp.",
        "java": str(JAVA), "javac": str(JAVAC), "javac_sha256": sha(JAVAC),
        "jarde_cli": str(JARDE), "jarde_cli_sha256": sha(JARDE),
        "jadx_cli": str(JADX), "jadx_cli_sha256": sha(JADX),
        "jadx_version_stdout": (OUT / "jadx-version.stdout").read_text() if (OUT / "jadx-version.stdout").exists() else None,
        "cases": cases,
        "commands": commands,
    }
    if not (OUT / "jadx-version.stdout").exists():
        run("jadx-version", [JADX, "--version"], OUT, OUT, commands)
        manifest["jadx_version_stdout"] = (OUT / "jadx-version.stdout").read_text(errors="replace")
    manifest["files"] = [
        {"path": str(path.relative_to(OUT)), "bytes": path.stat().st_size, "sha256": sha(path)}
        for path in sorted(OUT.rglob("*")) if path.is_file() and path.name != "manifest.json"
    ]
    (OUT / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({
        name: {flavor: {"compile": row.get("compile_exit"),
                        "runtime": row.get("runtime_stdout", "").strip()}
               for flavor, row in case["flavors"].items()}
        for name, case in ((c["probe"], c) for c in cases)
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
