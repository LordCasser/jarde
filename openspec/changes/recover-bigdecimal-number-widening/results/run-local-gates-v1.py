import json,subprocess,sys
from pathlib import Path
r=Path(__file__).resolve().parent
plan=json.loads((r/'local-gate-plan-v1.json').read_text())
for label,argv in plan:
 print('START '+label,flush=True)
 p=subprocess.run([sys.executable,str(r/'run-root-gate-v1.py'),label,*argv])
 if p.returncode:raise SystemExit(p.returncode)
print('ALL LOCAL GATES COMPLETE',flush=True)
