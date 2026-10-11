import sys,json,importlib.util,hashlib
from pathlib import Path
sys.dont_write_bytecode=True
P=Path('/private/tmp/jarde-cf11-own-ci-prep-luna-v5/verify-cf11-own-ci.py')
assert hashlib.sha256(P.read_bytes()).hexdigest()=='7669fb27ff653b5e8e35b68a2b54ec0723b4d5d63ee5c05fc0d7af5ed639c1f7'
s=importlib.util.spec_from_file_location('cf11_root_reviewed',P);v=importlib.util.module_from_spec(s);s.loader.exec_module(v)
product='b155877ee9a3b1894b053e70ffbf1cbf0a9512fa'
e,md,local=v.verify_workspace(product,Path('/private/tmp/jarde-cf11-full-workspace-root-v3'))
cli=v.verify_final_cli_build(product,e['source_pins'],Path('/private/tmp/jarde-cf11-side-exits-cli-build-root-v10/execution.json'),Path('/private/tmp/jarde-cf11-cli-build-root-v10.py'),'6ac9b86ad5ea5be20c0f65f0613fec14fead0c3d55b8340c5675e29b26c1ba3a',Path('/private/tmp/jarde-cf11-side-exits-candidate-cli-root-v10'),'09f4eac277ee9ddfd60a627f7b132877841f4859bb2c655fc21cb8d25ac197db')
required=v.verify_named_tests(local['target_observations'],product,e['source_pins'],md)
out=Path('/private/tmp/jarde-cf11-product-local-binding-root-v1.json');assert not out.exists()
result={'status':'accepted_git_local_cli_only','product_commit':product,'workspace':{k:a for k,a in local.items() if k!='target_observations'},'cli':cli,'required_tests':required,'own_product_ci_pending':True,'checker_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'product_commit':product,'target_count':local['target_count'],'source_count':len(e['source_pins']),'own_product_ci_pending':True}))
