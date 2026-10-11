"""Run the unchanged product verifier with strict unique public-name redaction binding."""
from pathlib import Path
import contextlib,hashlib,importlib.util,io,json,re,sys
SOURCE=Path('/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-conditional-switch-fallthrough/results/verify-conditional-ci-root-v5.py')
assert hashlib.sha256(SOURCE.read_bytes()).hexdigest()=='bea37c7d4d37182168a7c1cbbde3ee31c94d61f6bc32667bedbdd02ce7e5b0ca'
spec=importlib.util.spec_from_file_location('conditional_exact_product_v5',SOURCE)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
redactions=[]
def bind_public_name(raw,state,ref):
 if '***' not in raw:return raw
 pattern='^'+'[^\\n]*'.join(re.escape(part) for part in raw.split('***'))+'$'
 candidates=[name for name,status in ref['outcomes'] if status==state and re.fullmatch(pattern,name)]
 m.need(len(candidates)==1,'masked CI name does not uniquely bind a baseline outcome: '+repr((ref['key'],raw,state)))
 return candidates[0]
def parse_ci_workspace(group,seed,targets):
 m.need(seed in group,'CI command group missing configured seed')
 headers=[]
 for line in group.splitlines():
  if 'Running ' in m.clean(line):
   item=m.parse_ci_header(line,targets)
   if item is not None:headers.append(item)
 blocks=m.stdout_blocks_cargo_json(group.encode())
 m.need(len(headers)==len(targets) and len(blocks)==len(targets),'CI target header/result block counts do not match observed full target set')
 keys=[x['key'] for x in headers]
 m.need(len(keys)==len(set(keys)) and set(keys)=={x['key'] for x in targets},'CI target headers omit/duplicate/introduce target')
 observed=[];by_key={x['key']:x for x in targets}
 for target,block in zip(headers,blocks):
  ref=by_key[target['key']];bound=[]
  for raw,state in block['outcomes']:
   name=bind_public_name(raw,state,ref);bound.append((name,state))
   if name!=raw:redactions.append({'seed':seed,'target':list(ref['key']),'raw_public_label':raw,'bound_public_test_name':name,'status':state,'same_status_unique_candidate':True})
  m.need(len({name for name,_ in bound})==len(bound),'redaction creates duplicate bound outcome')
  m.need(block['counts']==ref['counts'] and tuple(bound)==ref['outcomes'],'CI named outcomes/summary differ from local target '+repr(ref['key']))
  observed.append({'package':ref['key'][0],'name':ref['key'][1],'kind':ref['key'][2],'source_path':ref['source_path'],'counts':list(block['counts']),'test_outcomes':[{'name':n,'status':state} for n,state in bound]})
 return {'seed':seed,'target_count':len(observed),'workspace_totals':[sum(x['counts'][i] for x in observed) for i in range(3)],'targets':observed}
m.parse_ci_workspace=parse_ci_workspace
output=io.StringIO()
with contextlib.redirect_stdout(output):m.main()
path=Path(sys.argv[sys.argv.index('--acceptance')+1]);doc=json.loads(path.read_text())
doc['ci_public_label_binding']={'wrapper':{'path':str(Path(__file__).resolve()),'sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()},'unchanged_product_verifier':{'path':str(SOURCE),'sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest()},'mappings':redactions,'rule':'Only *** in public CI labels may bind, to exactly one same-status name in the product-Git/live-pinned local target; no raw bytes are rewritten, all target/order/count/status checks remain exact.'}
doc['acceptance_scope']+=' GitHub-masked public test labels are uniquely bound to the same-status product-pinned target names; raw masked logs remain unchanged.'
path.write_text(json.dumps(doc,ensure_ascii=False,indent=2)+'\n')
summary=json.loads(output.getvalue());summary['public_label_redactions_uniquely_bound']=len(redactions);print(json.dumps(summary))
