from pathlib import Path
import re,shutil,subprocess,json,hashlib,tempfile,concurrent.futures
R=Path('/Users/lordcasser/workspace/projects/jarde')
E=R/'openspec/evidence/java-syntax-2026-09-22'
A=E/'deferred-evaluation/root-acceptance'
A.mkdir(exist_ok=True)
W=Path(tempfile.mkdtemp(prefix='jarde-root-deferred-accept-'))
cli=R/'target/debug/jarde-cli';before=hashlib.sha256(cli.read_bytes()).hexdigest()
rows=[('construction','construction-consumers/interleaved-effect'),('values','deferred-evaluation'),('checks','deferred-evaluation/checks'),('allocations','deferred-evaluation/allocations'),('structured','deferred-evaluation/structured-scopes')]
def replay(row):
 name,rel=row;src=E/rel;out=A/name;out.mkdir(exist_ok=True)
 for f in src.glob('*.java'):shutil.copy2(f,out/f.name)
 code=(src/'run_audit.py').read_text()
 old='openspec/evidence/java-syntax-2026-09-22/'+rel
 code=code.replace(old,str(out.relative_to(R)))
 code,n=re.subn(r"W=Path\('[^']+'\)",f"W=Path({str(W/name)!r})",code,count=1)
 assert n==1
 script=out/'run_audit.py';script.write_text(code)
 p=subprocess.run(['python3',str(script)],capture_output=True,text=True,timeout=150)
 (out/'replay.log').write_text(p.stdout+p.stderr)
 assert p.returncode==0,(name,p.stderr)
 s=json.loads((out/'summary.json').read_text())
 orig=(out/'original.txt').read_text().splitlines()
 got=(out/'jarde.txt').read_text().splitlines() if (out/'jarde.txt').exists() else []
 s['jarde_same_complete_output']=len(orig)==len(got) and orig==got
 s['actual_original_lines']=len(orig)
 s['cli_sha256']=before
 (out/'root-summary.json').write_text(json.dumps(s,indent=2)+'\n')
 return name,{k:s[k]for k in ['cases','jarde_quotes','jarde_javac','jarde_same_complete_output','actual_original_lines','jadx_equal']if k in s}
with concurrent.futures.ThreadPoolExecutor(max_workers=3) as ex: results=dict(ex.map(replay,rows))
assert before==hashlib.sha256(cli.read_bytes()).hexdigest()
(A/'summary.json').write_text(json.dumps({'cli_sha256':before,'groups':results},indent=2)+'\n')
(A/'run_audits.py').write_text(Path(__file__).read_text())
print(json.dumps(results,indent=2))
