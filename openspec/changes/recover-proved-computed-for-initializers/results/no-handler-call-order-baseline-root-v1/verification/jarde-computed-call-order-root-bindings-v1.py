from pathlib import Path
import json, hashlib
root=Path('/private/tmp/jarde-computed-call-order-baseline-root-v2')
x=json.loads((root/'execution.json').read_text())
by={c['index']:c for c in x['commands']}
verified=[]
for prof in x['profiles']:
 base=100 if prof['jdk']=='jdk8' else 118
 tool={n:p['path'] for n,p in prof['tools'].items()}
 for c in x['commands'][base-100:base-100+18]:
  n=Path(c['argv'][0]).name
  if n in tool: assert c['argv'][0]==tool[n]
 b=root/prof['jdk']; src=Path(x['source_pins'][0]['path']).parent
 assert by[base]['argv']==[tool['javac'],'-encoding','UTF-8','-g:none','-source','8','-target','8','-proc:none','-d',str(b/'original'),str(src/'ComputedForEffectControl.java'),str(src/'Runner.java')]
 for variant in ('original','pending'):
  assert prof['runtimes'][variant]==by[prof['runtimes'][variant]['index']]
 for render in prof['renders']:
  run=render['runtime']; assert run==by[run['index']]
  compile=by[run['index']-1]
  source=Path(render['source']['path'])
  assert compile['argv']==[tool['javac'],'-encoding','UTF-8','-source','8','-target','8','-proc:none','-d',str(source.parent),str(source)]
  assert by[run['index']-2]['streams']['stdout']==render['report']
  verified.append({'jdk':prof['jdk'],'variant':render['variant'],'mode':render['mode'],'compile_index':compile['index'],'runtime_index':run['index']})
y={'status':'accepted','scope':'additional root exact tool/compile/record bindings for current while baseline','execution_sha256':hashlib.sha256((root/'execution.json').read_bytes()).hexdigest(),'all_profile_runtime_rows_equal_recorded_commands':True,'all_java_tools_match_profile_pins':True,'exact_original_source_compile_argv_verified':True,'render_compile_runtime_bindings':verified,'generated_class_pins_timing':'post-run observed; no pre-runtime inventory claim'}
out=Path('/private/tmp/jarde-computed-call-order-root-bindings-acceptance-v1.json');out.write_text(json.dumps(y,indent=2)+'\n')
print('accepted: 2 original compiles, 8 generated compiles, 12 runtime bindings, profile tool identities')
