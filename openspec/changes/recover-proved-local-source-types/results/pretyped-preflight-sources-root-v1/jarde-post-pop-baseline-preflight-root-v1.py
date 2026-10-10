import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
PRIVATE = Path('/private/tmp/jarde-cf12-post-pop-baseline-luna-v2')

def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module

c = load('post_pop_collector_review_v2', PRIVATE/'collect-post-pop-baseline-v2.py')
v = load('post_pop_verifier_review_v2', PRIVATE/'verify-post-pop-baseline-v2.py')
c.verify_historical_inputs()
print('Historical pinned inputs and original raw streams verified without invoking any toolchain.')
method_count = 0
for case in sorted(v.CASES):
    for p in sorted((v.CF12/'render-root-v1'/case).glob('*/jarde-all.stdout.raw')):
        doc = json.loads(p.read_bytes())
        bytecode = v.javap_instructions(p.parent/'javap.stdout.raw')
        for key, (entry, report) in v.report_map(doc).items():
            assert key in bytecode, (case, p.parent.name, key)
            physical = {bci for bci, opcode in bytecode[key]}
            for segment in report['source_map']['segments']:
                for origin in v.origins(segment):
                    assert origin['method'] == entry['item']['identity']
                    assert origin['bci'] in physical, (case, key, origin['bci'])
            method_count += 1
print(json.dumps({'historical_methods_with_exact_physical_origin_parser':method_count}))
c.verify_pop_freeze(str(c.CLI_FIXED), str(c.META), str(c.BUILD), c.SOURCE_BASE)
print('Pop freeze preflight verified.')
