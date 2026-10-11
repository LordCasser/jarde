import sys, importlib.util, json
from pathlib import Path
sys.dont_write_bytecode=True
p=Path('/private/tmp/jarde-constructor-conditional-own-ci-verifier-root-v1.py');s=importlib.util.spec_from_file_location('v',p);v=importlib.util.module_from_spec(s);s.loader.exec_module(v)
product='c956050611f63a06f57caf0b1280238e81faeb80';ci=v.load_ci_helper();refs=v.verify_references(product,ci)
ex,meta,local=v.verify_workspace(product,v.EXPECTED_WORKSPACE,ci)
required=v.verify_required_tests(local['target_observations'],product,ex['source_pins_before'],meta,ci)
build=v.verify_final_build(product,ex['source_pins_before'],v.EXPECTED_BUILD_RECORD,v.EXPECTED_FINAL_CLI,ci)
print(json.dumps({'status':'accepted-local-product-binding-only','product':product,'target_count':local['target_count'],'required_tests':required['required_count'],'build':build,'ci_pending':True}))
