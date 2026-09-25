from __future__ import annotations

import difflib
import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
OUT = ROOT / "openspec/evidence/java-syntax-2026-09-22/shifts"
WORK = Path("/tmp/jarde-shifts-audit")
CLI = ROOT / "target/debug/jarde-cli"
OUT.mkdir(parents=True, exist_ok=True)
WORK.mkdir(parents=True, exist_ok=True)


SOURCES = {
    "ShiftAudit": r'''public class ShiftAudit {
 public static int left(int value,int distance){return value << distance;}
 public static int right(int value,int distance){return value >> distance;}
 public static int unsigned(int value,int distance){return value >>> distance;}
 public static long leftLong(long value,int distance){return value << distance;}
 public static long rightLong(long value,int distance){return value >> distance;}
 public static long unsignedLong(long value,int distance){return value >>> distance;}
 public static int nested(int value,int distance){return (value << distance) + (value >>> (distance + 1)) * 2;}
 public static int local(int value,int distance){int shifted=value << distance;return shifted ^ (shifted >>> 3);}
 public static int branch(int value,int distance,boolean choose){int shifted=value << distance;if(choose)return shifted;return value >> distance;}
 public static int byteChar(byte value,char other,int distance){return (value << distance) | (other >>> distance);}
 public static int ordered(boolean failLeft,boolean failRight,int value,int distance){return ShiftEffects.left(failLeft,value) << ShiftEffects.right(failRight,distance);}
}''',
    "ShiftDistanceBoundary": r'''public class ShiftDistanceBoundary {
 public static int intLeftLongDistance(int value,long distance){return value << (int) distance;}
 public static int intRightLongDistance(int value,long distance){return value >> (int) distance;}
 public static int intUnsignedLongDistance(int value,long distance){return value >>> (int) distance;}
 public static int intLongDistanceLocal(int value,long distance){int shifted=value << (int) distance;return shifted;}
}''',
    "ShiftEffects": r'''public class ShiftEffects {
 public static int trace;
 public static int left(boolean fail,int value){trace=trace*10+1;if(fail)throw new IllegalStateException("left");return value;}
 public static int right(boolean fail,int value){trace=trace*10+2;if(fail)throw new IllegalArgumentException("right");return value;}
}''',
    "ShiftBasicRunner": r'''public class ShiftBasicRunner {
 private static void reset(){ShiftEffects.trace=0;}
 private static void out(String name,Object value){System.out.println(name+"="+value);}
 public static void main(String[] args){
  int[] ints={Integer.MIN_VALUE,-12345,-1,0,1,12345,Integer.MAX_VALUE};
  long[] longs={Long.MIN_VALUE,-1L,0L,1L,Long.MAX_VALUE};
  int[] distances={-65,-33,-1,0,1,31,32,63,64,65,127};
  for(int i=0;i<ints.length;i++)for(int d:distances){
   out("iL:"+i+":"+d,ShiftAudit.left(ints[i],d));
   out("iR:"+i+":"+d,ShiftAudit.right(ints[i],d));
   out("iU:"+i+":"+d,ShiftAudit.unsigned(ints[i],d));
   out("nested:"+i+":"+d,ShiftAudit.nested(ints[i],d));
   out("local:"+i+":"+d,ShiftAudit.local(ints[i],d));
   out("branchT:"+i+":"+d,ShiftAudit.branch(ints[i],d,true));
   out("branchF:"+i+":"+d,ShiftAudit.branch(ints[i],d,false));
  }
  for(int i=0;i<longs.length;i++)for(int d:distances){
   out("jL:"+i+":"+d,ShiftAudit.leftLong(longs[i],d));
   out("jR:"+i+":"+d,ShiftAudit.rightLong(longs[i],d));
   out("jU:"+i+":"+d,ShiftAudit.unsignedLong(longs[i],d));
  }
  for(byte value:new byte[]{Byte.MIN_VALUE,-1,0,1,Byte.MAX_VALUE})for(char other:new char[]{0,1,65535}){
   out("byteChar:"+value+":"+(int)other,ShiftAudit.byteChar(value,other,3));
  }
  for(boolean left:new boolean[]{false,true})for(boolean right:new boolean[]{false,true}){
   reset();try{out("ordered:"+left+":"+right,ShiftAudit.ordered(left,right,3,1)+":"+ShiftEffects.trace);}
   catch(RuntimeException error){out("ordered:"+left+":"+right,error.getClass().getName()+":"+error.getMessage()+":"+ShiftEffects.trace);}
  }
 }
}''',
    "ShiftDistanceRunner": r'''public class ShiftDistanceRunner {
 private static void out(String name,Object value){System.out.println(name+"="+value);}
 public static void main(String[] args){
  for(int value:new int[]{Integer.MIN_VALUE,-1,0,1,Integer.MAX_VALUE})for(long distance:new long[]{-65L,-1L,0L,1L,31L,32L,63L,64L,65L,Long.MAX_VALUE}){
   out("bL:"+value+":"+distance,ShiftDistanceBoundary.intLeftLongDistance(value,distance));
   out("bR:"+value+":"+distance,ShiftDistanceBoundary.intRightLongDistance(value,distance));
   out("bU:"+value+":"+distance,ShiftDistanceBoundary.intUnsignedLongDistance(value,distance));
   out("bLocal:"+value+":"+distance,ShiftDistanceBoundary.intLongDistanceLocal(value,distance));
  }
 }
}''',
}


def run(args: list[str], log_name: str, cwd: Path | None = None) -> int:
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)
    (OUT / log_name).write_text(result.stdout + result.stderr)
    return result.returncode


def write_sources() -> None:
    source_dir = WORK / "source"
    source_dir.mkdir(exist_ok=True)
    for name, text in SOURCES.items():
        (source_dir / f"{name}.java").write_text(text + "\n")
        (OUT / f"{name}.java").write_text(text + "\n")


def lines(path: Path) -> list[str]:
    return path.read_text().splitlines()


def diff_count(left: Path, right: Path) -> int:
    return sum(1 for line in difflib.ndiff(lines(left), lines(right)) if line.startswith(("+ ", "- ")))


def save_output_diff(prefix: str, variant: str, compile_status: object) -> None:
    original = OUT / f"{prefix}-original.txt"
    recovered = OUT / f"{prefix}-{variant}.txt"
    destination = OUT / f"{prefix}-{variant}-diff.txt"
    if not recovered.exists():
        destination.write_text(f"{variant} execution not run: javac exit code {compile_status}\n")
        return
    destination.write_text("".join(difflib.unified_diff(
        original.read_text().splitlines(keepends=True),
        recovered.read_text().splitlines(keepends=True),
        fromfile=f"{prefix}-original.txt", tofile=f"{prefix}-{variant}.txt",
    )))


def class_source(class_name: str, class_file: Path, prefix: str, support_names: list[str], runner_name: str) -> dict[str, object]:
    raw = subprocess.run(
        [str(CLI), "class-source", "--input", str(class_file), "--class", class_name,
         "--policy", "single-class", "--release", "8", "--format", "text"],
        capture_output=True, text=True, timeout=90,
    )
    (OUT / f"{prefix}-jarde.java.txt").write_text(raw.stdout)
    (OUT / f"{prefix}-jarde-report.txt").write_text(raw.stderr)
    recovered = WORK / prefix / "jarde"
    recovered.mkdir(parents=True, exist_ok=True)
    (recovered / f"{class_name}.java").write_text(raw.stdout)
    result: dict[str, object] = {
        "jarde_returncode": raw.returncode,
        "jarde_quotes": raw.stdout.count("@bytecode"),
    }
    support_sources = [str(WORK / "source" / f"{name}.java") for name in support_names]
    result["jarde_javac"] = run(
        ["javac", "--release", "8", "-d", str(recovered / "classes"),
         str(recovered / f"{class_name}.java"),
         *support_sources],
        f"{prefix}-jarde-javac.log",
    )
    if result["jarde_javac"] == 0:
        result["jarde_run"] = run(
            ["java", "-Xverify:all", "-cp", str(recovered / "classes"), runner_name],
            f"{prefix}-jarde.txt",
        )
        result["jarde_equal"] = (OUT / f"{prefix}-original.txt").read_bytes() == (OUT / f"{prefix}-jarde.txt").read_bytes()
    return result


def jadx_source(class_name: str, class_file: Path, prefix: str, support_names: list[str], runner_name: str) -> dict[str, object]:
    jadx_dir = WORK / prefix / "jadx"
    result: dict[str, object] = {"jadx": run(["jadx", "--no-res", "-d", str(jadx_dir), str(class_file)], f"{prefix}-jadx.log")}
    generated = next(jadx_dir.rglob(f"{class_name}.java"))
    generated_text = generated.read_text()
    (OUT / f"{prefix}-jadx.java.txt").write_text(generated_text)
    package = next((x for x in generated_text.splitlines() if x.startswith("package ")), "")
    support = WORK / prefix / "jadx-support"
    support.mkdir(parents=True, exist_ok=True)
    for name in support_names:
        (support / f"{name}.java").write_text(package + "\n" + SOURCES[name] + "\n")
    support_sources = [str(support / f"{name}.java") for name in support_names]
    result["jadx_javac"] = run(
        ["javac", "--release", "8", "-d", str(WORK / prefix / "jadx-classes"), str(generated),
         *support_sources],
        f"{prefix}-jadx-javac.log",
    )
    if result["jadx_javac"] == 0:
        package_prefix = package[len("package "):].rstrip(";") + "." if package else ""
        result["jadx_run"] = run(
            ["java", "-Xverify:all", "-cp", str(WORK / prefix / "jadx-classes"), package_prefix + runner_name],
            f"{prefix}-jadx.txt",
        )
        result["jadx_equal"] = (OUT / f"{prefix}-original.txt").read_bytes() == (OUT / f"{prefix}-jadx.txt").read_bytes()
    return result


def audit(class_name: str, prefix: str, source_names: list[str], runner_name: str) -> dict[str, object]:
    source_dir = WORK / "source"
    original = WORK / prefix / "original"
    original.mkdir(parents=True, exist_ok=True)
    inputs = [str(source_dir / f"{name}.java") for name in source_names]
    summary: dict[str, object] = {
        "class": class_name,
        "cli_sha256": hashlib.sha256(CLI.read_bytes()).hexdigest(),
        "source_javac": run(["javac", "--release", "8", "-g:none", "-d", str(original)] + inputs, f"{prefix}-source-javac.log"),
    }
    class_file = original / f"{class_name}.class"
    if summary["source_javac"] != 0:
        return summary
    summary["class_bytes"] = class_file.stat().st_size
    summary["class_sha256"] = hashlib.sha256(class_file.read_bytes()).hexdigest()
    summary["original_run"] = run(["java", "-Xverify:all", "-cp", str(original), runner_name], f"{prefix}-original.txt")
    run(["javap", "-c", "-v", "-p", str(class_file)], f"{prefix}-javap.txt")
    support_names = [name for name in source_names if name != class_name]
    summary.update(class_source(class_name, class_file, prefix, support_names, runner_name))
    summary.update(jadx_source(class_name, class_file, prefix, support_names, runner_name))
    summary["cases"] = len(lines(OUT / f"{prefix}-original.txt"))
    if (OUT / f"{prefix}-jarde.txt").exists():
        summary["jarde_diff_lines"] = diff_count(OUT / f"{prefix}-original.txt", OUT / f"{prefix}-jarde.txt")
    if (OUT / f"{prefix}-jadx.txt").exists():
        summary["jadx_diff_lines"] = diff_count(OUT / f"{prefix}-original.txt", OUT / f"{prefix}-jadx.txt")
    save_output_diff(prefix, "jarde", summary["jarde_javac"])
    save_output_diff(prefix, "jadx", summary["jadx_javac"])
    return summary


def main() -> None:
    write_sources()
    summaries = [
        audit("ShiftAudit", "basic", ["ShiftAudit", "ShiftEffects", "ShiftBasicRunner"], "ShiftBasicRunner"),
        audit("ShiftDistanceBoundary", "distance-boundary", ["ShiftDistanceBoundary", "ShiftDistanceRunner"], "ShiftDistanceRunner"),
    ]
    (OUT / "summary.json").write_text(json.dumps(summaries, indent=2) + "\n")
    (OUT / "run_audit.py").write_text(Path(__file__).read_text())
    print(json.dumps(summaries, indent=2))


if __name__ == "__main__":
    main()
