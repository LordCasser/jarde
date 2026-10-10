#!/usr/bin/env python3
"""Pure AST helper/schema preflight over closed historical docs; no external tools."""
import ast, hashlib, json, re
from pathlib import Path
ROOT=Path('/Users/lordcasser/workspace/projects/jarde'); E=ROOT/'openspec/evidence/java-syntax-2026-10-10'; HERE=Path('/private/tmp/recover-prefixed-one-arm-loops-replay-v1')
CONF={
'postfix':{'base':E/'em23-variable-postfix-loop/baseline-root-v1','src':E/'em23-variable-postfix-loop/results/prepare-baseline-luna-v1.py','srcsha':'2e35326ebe7b2b60bd47b60419375088bc36142e053ea41f086f3cf534743089','cls':'VariablePostfixLoop','keys':[('<init>','()V'),('countEmpty','(Ljava/util/List;)I')],'doc':'cases/javac23-jarde-render/class-source-default.json'},
'plain':{'base':E/'one-arm-loop-controls/baseline-root-v1','src':E/'one-arm-loop-controls/prepare-baseline-luna-v1.py','srcsha':'a1ed035789f96f2a945bc9e8499bb4d1d089639082ccb59ce61933c97660879c','cls':'PlainOneArmLoops','keys':[('<init>','()V'),('prefixWhile','(ZI)I'),('noPrefix','(ZI)I'),('loopAndTail','(ZI)I'),('takenArm','(ZI)I')],'doc':'cases/javac23-jarde-render/class-source-default.json'}}
def sha(b):return hashlib.sha256(b).hexdigest()
class Digest:
 def __init__(self,v):self.v=v
 def hexdigest(self):return self.v
result={'schema':'recover-prefixed-one-arm-loops-helper-schema-preflight-root-v1','tools_invoked':[],'suites':{},'blake3_boundary':'frozen source-map class digest is injected only to exercise pure helper schema because this Python runtime has no blake3 module'}
for name,c in CONF.items():
 raw=c['src'].read_bytes()
 if sha(raw)!=c['srcsha']:raise SystemExit(name+' baseline helper pin mismatch')
 doc=json.loads((c['base']/c['doc']).read_bytes()); class_digest=doc['class']['class_bytes']['digest']
 class_path=c['base']/f"cases/javac23-original/classes/{c['cls']}.class"; original=class_path.read_bytes()
 tree=ast.parse(raw); wanted={'package_of','package_of_text','parse_javap','parse_javap_methods','method_key','verify_report_map','b3','sha'}
 nodes=[n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name in wanted]
 ns={'__name__':'_pure_helpers_'+name,'Path':Path,'re':re,'hashlib':hashlib,'CLASS_NAME':c['cls'],'blake3':lambda data:Digest(class_digest)}
 exec(compile(ast.fix_missing_locations(ast.Module(body=nodes,type_ignores=[])),str(c['src']),'exec'),ns)
 ns['METHOD_KEYS']=c['keys']
 if name=='postfix':ns['METHODS']=set(c['keys'])
 else:ns['METHOD_FLAGS']={'<init>()V':1,'prefixWhile(ZI)I':9,'noPrefix(ZI)I':9,'loopAndTail(ZI)I':9,'takenArm(ZI)I':9}
 javap=(c['base']/f'cases/javac23-original/javap.txt').read_text()
 parser=ns.get('parse_javap',ns.get('parse_javap_methods')); parsed=parser(javap)
 fact={'collector_sha256':sha(raw),'class_file_sha256':sha(original),'method_parser_schema':type(parsed).__name__,'parsed_method_count':len(parsed),'package_of_text_present':'package_of_text' in ns}
 if name=='postfix':
  try: ns['verify_report_map'](doc,class_digest,original,parsed,False); fact['map_helper_result']='unexpectedly returned on historical fallback'; fact['expected_fallback_rejection']=False
  except AssertionError as e:
   outcomes={ns['method_key'](x['item']):(x['outcome'].get('kind'),x['outcome'].get('report',{}).get('quality')) for x in doc['methods']}
   fact['historical_method_outcomes']={str(k):{'outcome':v[0],'quality':v[1]} for k,v in outcomes.items()}
   fact['map_helper_result']=f'AssertionError at helper structured-report gate; historical outcomes={fact["historical_method_outcomes"]}'
   fact['expected_fallback_rejection']=any(v[0]!='recovered' or v[1]!='structured' for v in outcomes.values())
   if not fact['expected_fallback_rejection']:raise
 else:
  mapped=ns['verify_report_map'](doc,original,parsed,'default')
  fact['map_helper_result_keys']=sorted(mapped); fact['method_fact_keys']=sorted(mapped['method_presentation_facts'][0]); fact['historical_map_facts']=[{'method':x['method'],'outcome_kind':x['outcome_kind'],'bci_coverage_complete':x['bci_coverage_complete'],'origins_valid':x['origin_bindings_and_bcis_valid']} for x in mapped['method_presentation_facts']]
 result['suites'][name]=fact
out=HERE/'helper-schema-preflight-root-v1.json'; out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(result,ensure_ascii=False,indent=2))
