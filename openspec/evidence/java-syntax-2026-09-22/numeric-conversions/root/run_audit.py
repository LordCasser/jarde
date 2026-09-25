from pathlib import Path
import hashlib
import json
import re
import shutil
import subprocess


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
OUT = ROOT / "openspec/evidence/java-syntax-2026-09-22/numeric-conversions/root"
WORK = Path("/tmp/jarde-root-numeric-conversions-0626")
CLI = ROOT / "target/debug/jarde-cli"

if WORK.exists():
    raise SystemExit(f"refusing to overwrite existing work directory: {WORK}")
WORK.mkdir(parents=True)
OUT.mkdir(parents=True, exist_ok=True)

sources = {
    "ConversionSupport": """public class ConversionSupport {
    public static int take(byte x) { return 100 + x; }
    public static int take(short x) { return 200 + x; }
    public static int take(char x) { return 300 + x; }
    public static int take(int x) { return 400 + x; }
    public static long take(long x) { return 500L + x; }
    public static double take(float x) { return 600.0d + x; }
    public static double take(double x) { return 700.0d + x; }
}
""",
    "ConversionIntegers": """public class ConversionIntegers {
    public static long i2l(int x) { return (long) x; }
    public static float i2f(int x) { return (float) x; }
    public static double i2d(int x) { return (double) x; }
    public static byte i2b(int x) { return (byte) x; }
    public static char i2c(int x) { return (char) x; }
    public static short i2s(int x) { return (short) x; }
    public static int byteOverload(int x) { return ConversionSupport.take((byte) x); }
    public static int shortOverload(int x) { return ConversionSupport.take((short) x); }
    public static int charOverload(int x) { return ConversionSupport.take((char) x); }
}
""",
    "ConversionLongFloat": """public class ConversionLongFloat {
    public static int l2i(long x) { return (int) x; }
    public static float l2f(long x) { return (float) x; }
    public static double l2d(long x) { return (double) x; }
    public static int f2i(float x) { return (int) x; }
    public static long f2l(float x) { return (long) x; }
    public static double f2d(float x) { return (double) x; }
    public static int longToIntOverload(long x) { return ConversionSupport.take((int) x); }
    public static double longToFloatOverload(long x) { return ConversionSupport.take((float) x); }
    public static double floatToDoubleOverload(float x) { return ConversionSupport.take((double) x); }
}
""",
    "ConversionDouble": """public class ConversionDouble {
    public static int d2i(double x) { return (int) x; }
    public static long d2l(double x) { return (long) x; }
    public static float d2f(double x) { return (float) x; }
    public static int doubleToIntOverload(double x) { return ConversionSupport.take((int) x); }
}
""",
    "ConversionRunner": """public class ConversionRunner {
    private static String floatBits(float value) {
        return Integer.toHexString(Float.floatToRawIntBits(value));
    }

    private static String doubleBits(double value) {
        return Long.toHexString(Double.doubleToRawLongBits(value));
    }

    public static void main(String[] args) {
        int[] ints = { Integer.MIN_VALUE, -65537, -1, 0, 1, 65535, Integer.MAX_VALUE };
        long[] longs = { Long.MIN_VALUE, -4294967297L, -4294967296L, -1L, 0L, 1L, 4294967296L, 4294967297L, Long.MAX_VALUE };
        float[] floats = { Float.NEGATIVE_INFINITY, -Float.MAX_VALUE, -0.0f, 0.0f, Float.MIN_VALUE, 1.5f, Float.MAX_VALUE, Float.NaN, Float.POSITIVE_INFINITY };
        double[] doubles = { Double.NEGATIVE_INFINITY, -Double.MAX_VALUE, -0.0d, 0.0d, Double.MIN_VALUE, 1.5d, Double.MAX_VALUE, Double.NaN, Double.POSITIVE_INFINITY };

        for (int i = 0; i < ints.length; i++) {
            int x = ints[i];
            System.out.println("I:" + i + ":i2l=" + ConversionIntegers.i2l(x));
            System.out.println("I:" + i + ":i2f=" + floatBits(ConversionIntegers.i2f(x)));
            System.out.println("I:" + i + ":i2d=" + doubleBits(ConversionIntegers.i2d(x)));
            System.out.println("I:" + i + ":i2b=" + ConversionIntegers.i2b(x));
            System.out.println("I:" + i + ":i2c=" + (int) ConversionIntegers.i2c(x));
            System.out.println("I:" + i + ":i2s=" + ConversionIntegers.i2s(x));
            System.out.println("I:" + i + ":byteOverload=" + ConversionIntegers.byteOverload(x));
            System.out.println("I:" + i + ":shortOverload=" + ConversionIntegers.shortOverload(x));
            System.out.println("I:" + i + ":charOverload=" + ConversionIntegers.charOverload(x));
        }

        for (int i = 0; i < longs.length; i++) {
            long x = longs[i];
            System.out.println("L:" + i + ":l2i=" + ConversionLongFloat.l2i(x));
            System.out.println("L:" + i + ":l2f=" + floatBits(ConversionLongFloat.l2f(x)));
            System.out.println("L:" + i + ":l2d=" + doubleBits(ConversionLongFloat.l2d(x)));
            System.out.println("L:" + i + ":longToIntOverload=" + ConversionLongFloat.longToIntOverload(x));
            System.out.println("L:" + i + ":longToFloatOverload=" + doubleBits(ConversionLongFloat.longToFloatOverload(x)));
        }

        for (int i = 0; i < floats.length; i++) {
            float x = floats[i];
            System.out.println("F:" + i + ":f2i=" + ConversionLongFloat.f2i(x));
            System.out.println("F:" + i + ":f2l=" + ConversionLongFloat.f2l(x));
            System.out.println("F:" + i + ":f2d=" + doubleBits(ConversionLongFloat.f2d(x)));
            System.out.println("F:" + i + ":floatToDoubleOverload=" + doubleBits(ConversionLongFloat.floatToDoubleOverload(x)));
        }

        for (int i = 0; i < doubles.length; i++) {
            double x = doubles[i];
            System.out.println("D:" + i + ":d2i=" + ConversionDouble.d2i(x));
            System.out.println("D:" + i + ":d2l=" + ConversionDouble.d2l(x));
            System.out.println("D:" + i + ":d2f=" + floatBits(ConversionDouble.d2f(x)));
            System.out.println("D:" + i + ":doubleToIntOverload=" + ConversionDouble.doubleToIntOverload(x));
        }
    }
}
""",
}

for name, source in sources.items():
    (WORK / f"{name}.java").write_text(source, encoding="utf-8")
    (OUT / "sources").mkdir(exist_ok=True)
    (OUT / "sources" / f"{name}.java").write_text(source, encoding="utf-8")


def run(args, output):
    result = subprocess.run(args, capture_output=True, text=True, timeout=60)
    (output).write_text(result.stdout + result.stderr, encoding="utf-8")
    return result


def write_sha(path, output):
    output.write_text(hashlib.sha256(path.read_bytes()).hexdigest() + "  " + path.name + "\n", encoding="utf-8")


cli_before = hashlib.sha256(CLI.read_bytes()).hexdigest()
(OUT / "cli-sha-before.txt").write_text(cli_before + "  jarde-cli\n", encoding="utf-8")

all_sources = [str(WORK / f"{name}.java") for name in sources]
original = WORK / "original"
original.mkdir()
original_compile = run(["javac", "--release", "8", "-g:none", "-d", str(original), *all_sources], OUT / "original-javac.log")
if original_compile.returncode:
    raise SystemExit("original javac failed; see original-javac.log")

run(["java", "-Xverify:all", "-cp", str(original), "ConversionRunner"], OUT / "original.txt")

groups = ["ConversionIntegers", "ConversionLongFloat", "ConversionDouble"]
summary = {
    "java": "23.0.1",
    "release": 8,
    "cli_sha256_before": cli_before,
    "groups": {},
    "sources": list(sources),
}

# JADX is run once over the complete source class set.  Its default-package classes are emitted
# under `defpackage`, so compiling only one generated class would make the shared runner fail to
# resolve the other two classes and would confuse a toolchain failure with a semantic result.
jadx_all = WORK / "jadx-all"
jadx_result = run(["jadx", "--no-res", "-d", str(jadx_all), *(str(original / f"{name}.class") for name in groups)], OUT / "jadx.log")
jadx_sources = {}
if jadx_result.returncode == 0:
    for name in groups:
        generated = next(jadx_all.rglob(f"{name}.java"), None)
        if generated is not None:
            jadx_sources[name] = generated

for name in groups:
    group = OUT / name
    group.mkdir(exist_ok=True)
    class_path = original / f"{name}.class"
    write_sha(class_path, group / "original-class-sha256.txt")
    run(["javap", "-c", "-v", str(class_path)], group / "javap.txt")
    jarde = subprocess.run(
        [str(CLI), "class-source", "--input", str(class_path), "--class", name, "--policy", "single-class", "--release", "8", "--format", "text"],
        capture_output=True,
        text=True,
        timeout=60,
    )
    (group / "jarde.java.txt").write_text(jarde.stdout, encoding="utf-8")
    (group / "jarde-report.txt").write_text(jarde.stderr, encoding="utf-8")
    jarde_dir = WORK / f"jarde-{name}"
    jarde_dir.mkdir()
    (jarde_dir / f"{name}.java").write_text(jarde.stdout, encoding="utf-8")
    jarde_compile = run(["javac", "--release", "8", "-g:none", "-cp", str(original), "-d", str(jarde_dir / "classes"), str(jarde_dir / f"{name}.java"), str(WORK / "ConversionSupport.java"), str(WORK / "ConversionRunner.java")], group / "jarde-javac.log")
    if jarde_compile.returncode == 0:
        run(["java", "-Xverify:all", "-cp", f"{jarde_dir / 'classes'}:{original}", "ConversionRunner"], group / "jarde.txt")
    else:
        (group / "jarde.txt").write_text("<not run: jarde javac failed>\n", encoding="utf-8")

    jadx_dir = WORK / f"jadx-{name}"
    generated = jadx_sources.get(name)
    shutil.copy2(OUT / "jadx.log", group / "jadx.log")
    if generated is None:
        (group / "jadx.java.txt").write_text("<no JADX source>\n", encoding="utf-8")
        jadx_compile_code = None
        jadx_run_code = None
    else:
        jadx_source = generated.read_text(encoding="utf-8")
        (group / "jadx.java.txt").write_text(jadx_source, encoding="utf-8")
        package = next((line for line in jadx_source.splitlines() if line.startswith("package ")), "")
        package_prefix = package[len("package "):].rstrip(";") + "." if package else ""
        support_dir = WORK / f"jadx-support-{name}"
        support_dir.mkdir()
        for support_name in ("ConversionSupport", "ConversionRunner"):
            (support_dir / f"{support_name}.java").write_text(package + "\n" + sources[support_name], encoding="utf-8")
        generated_sources = [str(jadx_sources[other]) for other in groups]
        jadx_compile = run(["javac", "--release", "8", "-g:none", "-cp", str(original), "-d", str(jadx_dir / "classes"), *generated_sources, str(support_dir / "ConversionSupport.java"), str(support_dir / "ConversionRunner.java")], group / "jadx-javac.log")
        jadx_compile_code = jadx_compile.returncode
        if jadx_compile.returncode == 0:
            jadx_run = run(["java", "-Xverify:all", "-cp", f"{jadx_dir / 'classes'}:{original}", package_prefix + "ConversionRunner"], group / "jadx.txt")
            jadx_run_code = jadx_run.returncode
        else:
            (group / "jadx.txt").write_text("<not run: JADX javac failed>\n", encoding="utf-8")
            jadx_run_code = None

    javap_text = (group / "javap.txt").read_text(encoding="utf-8")
    opcodes = {}
    for opcode in ("i2l", "i2f", "i2d", "l2i", "l2f", "l2d", "f2i", "f2l", "f2d", "d2i", "d2l", "d2f", "i2b", "i2c", "i2s"):
        opcodes[opcode] = len(re.findall(rf"^\s+\d+:\s+{opcode}\b", javap_text, re.MULTILINE))
    summary["groups"][name] = {
        "original_class_bytes": class_path.stat().st_size,
        "jarde_returncode": jarde.returncode,
        "jarde_quotes": jarde.stdout.count("@bytecode"),
        "jarde_javac": jarde_compile.returncode,
        "jadx_returncode": jadx_result.returncode,
        "jadx_javac": jadx_compile_code,
        "jadx_run": jadx_run_code,
        "code_attributes": len(re.findall(r"^\s+Code:$", javap_text, re.MULTILINE)),
        "opcodes_in_javap": opcodes,
    }

cli_after = hashlib.sha256(CLI.read_bytes()).hexdigest()
(OUT / "cli-sha-after.txt").write_text(cli_after + "  jarde-cli\n", encoding="utf-8")
summary["cli_sha256_after"] = cli_after
summary["cli_unchanged"] = cli_before == cli_after
(OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")

# Compare every successfully executed class-level output with the original runner.
original_lines = (OUT / "original.txt").read_text(encoding="utf-8").splitlines()
for name in groups:
    group = OUT / name
    for label in ("jarde", "jadx"):
        result_path = group / f"{label}.txt"
        if not result_path.exists() or result_path.read_text(encoding="utf-8").startswith("<not run"):
            continue
        lines = result_path.read_text(encoding="utf-8").splitlines()
        diffs = [f"line {index}: original={left!r} {label}={right!r}" for index, (left, right) in enumerate(zip(original_lines, lines), 1) if left != right]
        if len(lines) != len(original_lines):
            diffs.append(f"line-count: original={len(original_lines)} {label}={len(lines)}")
        (group / f"{label}-differences.txt").write_text("\n".join(diffs) + ("\n" if diffs else ""), encoding="utf-8")

print(json.dumps(summary, indent=2))
