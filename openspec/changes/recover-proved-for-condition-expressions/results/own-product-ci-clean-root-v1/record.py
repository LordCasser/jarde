import sys,subprocess,hashlib,json,datetime
from pathlib import Path
D=Path(sys.argv[1]);D.mkdir(parents=True,exist_ok=False)
argv=sys.argv[2:];assert argv
started=datetime.datetime.now(datetime.timezone.utc).isoformat();r=subprocess.run(argv,capture_output=True)
streams={}
for label,data in [('stdout',r.stdout),('stderr',r.stderr)]:
 p=D/(label+'.raw');p.write_bytes(data);streams[label]={'path':str(p),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}
e={'argv':argv,'exit_code':r.returncode,'started_at':started,'finished_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'streams':streams,'recorder_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
(D/'execution.json').write_text(json.dumps(e,indent=2)+'\n');print(json.dumps(e));print(r.stdout[:16384].decode(errors='replace'));print(r.stderr[:16384].decode(errors='replace'),file=sys.stderr);sys.exit(r.returncode)
