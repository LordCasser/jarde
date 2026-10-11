import hashlib,json,subprocess,sys
from pathlib import Path
sys.dont_write_bytecode=True
P=Path('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1')
REC=Path('/private/tmp/jarde-cf11-invocations-root-v1/record.py')
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
for name,h in [('workspace_runner.py','711fbd12ba12f8e559340f5dc8bcb3edd6a5dd6b075c9cf54677353565bc506e'),('accept_workspace.py','16683decff47cdae9cc73b7abadcbc72bdf1ce9a71733b05d5e715cbdb380df9'),]:assert H(P/name)==h
assert H('/private/tmp/jarde-hoisted-iterator-final-build-root-v2.py')=='ce22ee46288d21ec53514743c66b5c26fb7761746ffdb4f442ea31877100ecc3'
assert H('/private/tmp/jarde-hoisted-iterator-accept-final-root-v2.py')=='a71e63ac2150a1681b9b2ff8df7611335d6dbb033b0af37d74e7896d28701a92'
assert H('/private/tmp/jarde-hoisted-iterator-regressions-root-v6.py')=='b91e0ca6967691f709cee6d934d7f1749e0cb3c0ad4988000bb8d2826db385b2'
assert H('/private/tmp/jarde-hoisted-iterator-regressions-accept-root-v5.py')=='8e5c4efbee691bf23ccada07b791b36e5ff61d66d24ae411fba4b194455d6088'
assert H(REC)=='825c263039006ed7f92beaf1afede1c0ea1671da83d12a6bdb6625d55cf94f4e'
reg=Path('/private/tmp/jarde-hoisted-iterator-regressions-root-v6/execution.json')

def run(label,script,args):
 inv=Path('/private/tmp/jarde-hoisted-iterator-'+label+'-invocation-root-v4')
 subprocess.run(['python3',str(REC),str(inv),'python3',str(script),*args],check=True)
 assert json.loads((inv/'execution.json').read_text())['exit_code']==0
 return inv
run('regressions',Path('/private/tmp/jarde-hoisted-iterator-regressions-root-v6.py'),[])
run('regressions-acceptance',Path('/private/tmp/jarde-hoisted-iterator-regressions-accept-root-v5.py'),[H(reg)])
regargs=['--regression-receipt',str(reg),'--regression-receipt-sha256',H(reg)]
ws=P/'workspace-run-v4'
run('workspace',P/'workspace_runner.py',regargs+['--candidate-cli','/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v6','--out',str(ws)])
wa=P/'workspace-acceptance-v4.json'
run('workspace-acceptance',P/'accept_workspace.py',regargs+['--workspace',str(ws),'--out',str(wa)])
wargs=['--workspace-receipt',str(ws/'execution.json'),'--workspace-receipt-sha256',H(ws/'execution.json'),'--workspace-acceptance',str(wa),'--workspace-acceptance-sha256',H(wa)]
final=P/'final-build-v5'
run('final-build',Path('/private/tmp/jarde-hoisted-iterator-final-build-root-v2.py'),regargs+wargs+['--out',str(final)])
inv=run('final-acceptance',Path('/private/tmp/jarde-hoisted-iterator-accept-final-root-v2.py'),regargs+wargs+['--final-receipt',str(final/'execution.json'),'--final-receipt-sha256',H(final/'execution.json')])
fa=P/'final-acceptance-v5.json';assert not fa.exists();raw=(inv/'stdout.raw').read_bytes();assert json.loads(raw)['status']=='accepted_local_full_validation_only';fa.write_bytes(raw)
print(json.dumps({'status':'actual_local_validation_accepted','regression_sha256':H(reg),'workspace_sha256':H(ws/'execution.json'),'workspace_acceptance_sha256':H(wa),'final_sha256':H(final/'execution.json'),'final_acceptance_sha256':H(fa)}),flush=True)
