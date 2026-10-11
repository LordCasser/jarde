import json,subprocess,sys
from pathlib import Path
D=Path(__file__).parent
rows=[]
for job in json.loads((D/'jobs.json').read_text()):
 r=subprocess.run(job['argv'],capture_output=True);(D/(job['subject']+'.stdout.raw')).write_bytes(r.stdout);(D/(job['subject']+'.stderr.raw')).write_bytes(r.stderr);rows.append({'subject':job['subject'],'argv':job['argv'],'exit_code':r.returncode});(D/'execution.json').write_text(json.dumps(rows,indent=2)+'\n')
 if r.returncode: sys.exit(r.returncode)
print('both postfix collectors completed; independent checker pending')
