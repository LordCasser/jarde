import json,hashlib,subprocess,sys
from pathlib import Path
D=Path(__file__).parent
rows=[]
for subject in ('nested-loops4','side-exit-controls'):
 out=Path('/private/tmp/jarde-cf11-'+subject+'-postfix-root-v2')
 argv=['python3','/private/tmp/jarde-cf11-postfix-tools-root-v4/check_postfix.py','--execution',str(out/'execution.json'),'--out',str(out/'checker-root-v2.json')]
 r=subprocess.run(argv,capture_output=True)
 streams={}
 for label,data in [('stdout',r.stdout),('stderr',r.stderr)]:
  p=D/(subject+'-check.'+label+'.raw');p.write_bytes(data);streams[label]={'path':str(p),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}
 rows.append({'subject':subject,'argv':argv,'exit_code':r.returncode,'streams':streams})
 (D/'check-execution.json').write_text(json.dumps(rows,indent=2)+'\n')
 print(r.stdout.decode(errors='replace'));print(r.stderr.decode(errors='replace'),file=sys.stderr)
 if r.returncode:sys.exit(r.returncode)
