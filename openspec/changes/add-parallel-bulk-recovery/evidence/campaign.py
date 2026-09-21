import json, os, shutil, statistics, subprocess, time

REPO="/Users/lordcasser/workspace/projects/jarde"
V="/Users/lordcasser/workspace/vulnerability/vulhub"
RAW="/tmp/jarde-b10/raw.jsonl"
SUBJECTS={
 "bcprov":(f"{V}/weblogic/weak_password/decrypt/lib/bcprov-jdk15on-152.jar","plain-jar"),
 "s2-009":(f"{V}/struts2/s2-009/S2-009.war","explicit-classpath"),
}
def measure(cmd, tag, artifact, tool, config, out=None):
    t0=time.perf_counter()
    p=subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    wall=time.perf_counter()-t0
    size=os.path.getsize(out) if out and os.path.exists(out) else 0
    row=dict(tool=tool,artifact=artifact,config=config,wall_s=round(wall,3),exit=p.returncode,output_bytes=size)
    with open(RAW,"a") as fh: fh.write(json.dumps(row)+"\n")
    print(f"{tag:<34} {wall:7.2f}s exit={p.returncode} out={size/1e6:.1f}MB", flush=True)
    if out and os.path.exists(out): os.remove(out)
    return row

def jadx(jar, out, jobs=None):
    cmd=["/opt/homebrew/bin/jadx","-d",out,"--no-res"] + (["-j",str(jobs)] if jobs else []) + [jar]
    return cmd

reps=int(os.environ.get("REPS","3"))
for label,(jar,policy) in SUBJECTS.items():
    roots=f"/tmp/jarde-b10/roots/{label}.json"
    out=f"/tmp/jarde-b10/campaign-{label}.jsonl"
    for rep in range(reps):
        for tool,config in (("jadx","default6"),("jadx","j1"),("jarde","jobs1"),("jarde","jobs2"),("jarde","jobs4")):
            if tool=="jadx":
                d=f"/tmp/jarde-b10/jadx-{label}-{config}-{rep}"; shutil.rmtree(d,ignore_errors=True)
                row=measure(jadx(jar,d,None if config=="default6" else 1), f"jadx {label} {config} rep{rep}", label, tool, config)
                shutil.rmtree(d,ignore_errors=True)
            else:
                jobs=config.replace("jobs","")
                cmd=[f"{REPO}/target/release/jarde-cli","export","--input",jar,"--policy",policy,"--release","8",
                     "--jobs",jobs,"--scope",json.dumps({"kind":"artifact_tree","root_container":"root"}),
                     "--roots","@"+roots,"--output",out]
                measure(cmd, f"jarde {label} {config} rep{rep}", label, tool, config, out)
print("--- medians ---")
rows=[json.loads(l) for l in open(RAW)]
g={}
for r in rows: g.setdefault((r["artifact"],r["tool"],r["config"]),[]).append(r["wall_s"])
for k in sorted(g): print(f"{k[0]:<9}{k[1]:<7}{k[2]:<9} n={len(g[k])} median={statistics.median(g[k]):.2f}s min={min(g[k]):.2f} max={max(g[k]):.2f}")
