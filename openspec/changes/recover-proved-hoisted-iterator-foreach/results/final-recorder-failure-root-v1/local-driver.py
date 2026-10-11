import hashlib,json,subprocess,sys
from pathlib import Path
sys.dont_write_bytecode=True
P=Path('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1')
REC=Path('/private/tmp/jarde-cf11-invocations-root-v1/record.py')
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
for name,h in [('workspace_runner.py','711fbd12ba12f8e559340f5dc8bcb3edd6a5dd6b075c9cf54677353565bc506e'),('accept_workspace.py','16683decff47cdae9cc73b7abadcbc72bdf1ce9a71733b05d5e715cbdb380df9'),('final_build_runner.py','48af92704170947376c798a303ed9e8a492b2f55ef55b4762bc2b1439a203ccd'),('accept_final_build.py','ec3faca2ba168464990f2ec89ff381b283af9df6b5bb265e326e23aecd518401')]:assert H(P/name)==h
assert H(REC)=='825c263039006ed7f92beaf1afede1c0ea1671da83d12a6bdb6625d55cf94f4e'
reg=Path('/private/tmp/jarde-hoisted-iterator-regressions-root-v5/execution.json')
assert json.loads(reg.read_text())['status']=='passed_regressions_candidate_only'
def run(label,script,args):
 inv=Path('/private/tmp/jarde-hoisted-iterator-'+label+'-invocation-root-v3')
 subprocess.run(['python3',str(REC),str(inv),'python3',str(script),*args],check=True)
 assert json.loads((inv/'execution.json').read_text())['exit_code']==0
 return inv
regargs=['--regression-receipt',str(reg),'--regression-receipt-sha256',H(reg)]
ws=P/'workspace-run-v3'
run('workspace',P/'workspace_runner.py',regargs+['--candidate-cli','/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v5','--out',str(ws)])
wa=P/'workspace-acceptance-v3.json'
run('workspace-acceptance',P/'accept_workspace.py',regargs+['--workspace',str(ws),'--out',str(wa)])
wargs=['--workspace-receipt',str(ws/'execution.json'),'--workspace-receipt-sha256',H(ws/'execution.json'),'--workspace-acceptance',str(wa),'--workspace-acceptance-sha256',H(wa)]
final=P/'final-build-v3'
run('final-build',P/'final_build_runner.py',regargs+wargs+['--out',str(final)])
inv=run('final-acceptance',P/'accept_final_build.py',regargs+wargs+['--final-receipt',str(final/'execution.json'),'--final-receipt-sha256',H(final/'execution.json')])
fa=P/'final-acceptance-v3.json';assert not fa.exists();raw=(inv/'stdout.raw').read_bytes();assert json.loads(raw)['status']=='accepted_local_full_validation_only';fa.write_bytes(raw)
print(json.dumps({'status':'actual_local_validation_accepted','regression_sha256':H(reg),'workspace_sha256':H(ws/'execution.json'),'workspace_acceptance_sha256':H(wa),'final_sha256':H(final/'execution.json'),'final_acceptance_sha256':H(fa)}),flush=True)
