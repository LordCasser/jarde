from pathlib import Path
import hashlib,json,re
import blake3
root=Path('/Users/lordcasser/workspace/projects/jarde')
r=Path('/private/tmp/jarde-conditional-boundary-ir-root-v2')
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
e=json.loads((r/'execution.json').read_text());assert e['status']=='public-ir-observations-collected' and e['temporary_test_removed'] and e['frozen_sources_unchanged']
md=json.loads((root/'openspec/changes/recover-proved-local-source-types/results/candidate-cli-typed-root-v1.json').read_text())
pins={p:h for g in ['candidate_sources','test_sources','canonical_files'] for p,h in md[g].items()}
assert e['source_pins_before']==e['source_pins_after']==pins
for p,h in pins.items():assert sha(root/p)==h
for p,h in e['input_class_pins'].items():assert sha(p)==h
assert not (root/'crates/jarde-java/tests/__root_conditional_boundary_ir_observer.rs').exists()
assert sha(r/'observer-root.rs')==e['observer_source_sha256']
assert len(e['commands'])==1
cmd=e['commands'][0];assert cmd['exit_code']==0 and cmd['guard_stop'] is None
assert cmd['argv']==['cargo','test','-p','jarde-java','--test','__root_conditional_boundary_ir_observer','--locked','--','--nocapture']
assert cmd['peak_target_bytes']<=1024**3 and cmd['free_bytes_after']>=5*1024**3
for k,ref in cmd['streams'].items():
 p=Path(ref['path']);assert p.parent==r and sha(p)==ref['sha256'] and p.stat().st_size==ref['bytes']
raw=(r/'0.stdout.raw').read_text();assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw)==[('1','0','0')]
blocks=re.findall(r'^METHOD_BEGIN (.*?)\n(.*?)^METHOD_END (.*?)$',raw,re.M|re.S);assert len(blocks)==10

def catch_name(class_bytes, index):
 import struct
 count=struct.unpack_from('>H',class_bytes,8)[0];pos=10;cp={};i=1
 while i<count:
  tag=class_bytes[pos];pos+=1
  if tag==1:
   n=struct.unpack_from('>H',class_bytes,pos)[0];pos+=2;cp[i]=(tag,class_bytes[pos:pos+n]);pos+=n
  elif tag in (7,8,16,19,20):cp[i]=(tag,struct.unpack_from('>H',class_bytes,pos)[0]);pos+=2
  elif tag in (3,4,9,10,11,12,17,18):pos+=4
  elif tag in (5,6):pos+=8;i+=1
  elif tag==15:pos+=3
  else:raise AssertionError(('unknown CP tag',tag))
  i+=1
 assert cp[index][0]==7 and cp[cp[index][1]][0]==1
 return cp[cp[index][1]][1].decode('ascii')

profiles=[];seen=set()
for header,body,tail in blocks:
 hm=re.fullmatch(r'class_leg=(javac8|javac23) owner=ConditionalSwitchBoundaries name=(\w+) descriptor=(.+)',header);assert hm
 leg,name,descriptor=hm.groups();assert tail==f'class_leg={leg} name={name} descriptor={descriptor}'
 assert (leg,name) not in seen;seen.add((leg,name))
 inp=Path(f'/private/tmp/jarde-conditional-boundary-preflight-root-v2/cases/{leg}/original/classes/ConditionalSwitchBoundaries.class');digest=blake3.blake3(inp.read_bytes()).hexdigest()
 # The first analysis report contains this reader-derived physical method and full stages.
 report=body.split('PHYSICAL_CODE ',1)[0]
 assert digest in report and 'execution: Complete {' in report
 stages=re.findall(r'StageResult \{\s*stage: (\w+),\s*state: (\w+),',report);assert stages==[(s,'Completed') for s in ['RawFacts','RawCfg','LegacyNormalization','CanonicalCfg','Frame','Ssa']]
 identity=re.search(r'method: PhysicalMethodId \{.*?name: JvmBytes\(\s*\[([^]]*)\].*?descriptor: JvmBytes\(\s*\[([^]]*)\]',report,re.S);assert identity
 decode=lambda s:bytes(map(int,re.findall(r'\d+',s))).decode('ascii')
 assert decode(identity[1])==name and decode(identity[2])==descriptor
 physical=[(int(a),int(b,16),int(c)) for a,b,c in re.findall(r'^PHYSICAL_INSTRUCTION bci=(\d+) opcode=0x([0-9a-f]+) width=(\d+)',body,re.M)]
 typed=[(int(a),int(b,16),int(c)) for a,b,c in re.findall(r'^PHYSICAL_TYPED_INSTRUCTION bci=(\d+) opcode=0x([0-9a-f]+) effective_opcode=0x[0-9a-f]+ width=(\d+)',body,re.M)];assert physical==typed and physical
 assert physical[0][0]==0 and all(a[0]+a[2]==b[0] for a,b in zip(physical,physical[1:]))
 nodes=[(int(a),json.loads(p),int(b)) for a,p,b in re.findall(r'^CANONICAL_BLOCK bci=(\d+) path=(\[[^]]*\]) end_bci=(\d+)',body,re.M)];assert nodes and all(not p for a,p,b in nodes)
 assert 'CANONICAL completeness=Complete unreachable=[]' in body
 assert len({a for a,p,b in nodes})==len(nodes)
 for at,op,width in physical:assert sum(a<=at<b for a,p,b in nodes)==1,(leg,name,at)
 edges=[{'from':int(a),'from_path':json.loads(ap),'kind':kind,'to':int(b),'to_path':json.loads(bp)} for a,ap,kind,b,bp in re.findall(r'^CANONICAL_EDGE from=(\d+) from_path=(\[[^]]*\]) kind=(.*?) to=(\d+) to_path=(\[[^]]*\])$',body,re.M)]
 ids={a for a,p,b in nodes};assert all(x['from'] in ids and x['to'] in ids and not x['from_path'] and not x['to_path'] for x in edges)
 ssa_ids=[int(a) for a in re.findall(r'^SSA_BLOCK bci=(\d+) path=\[\]',body,re.M)];assert set(ssa_ids)==ids and len(ssa_ids)==len(ids)
 handlers=[(int(o),int(a),int(b),int(h),int(cp)) for o,a,b,h,cp in re.findall(r'^PHYSICAL_EXCEPTION_TABLE ExceptionHandlerFact \{ ordinal: (\d+), start_bci: (\d+), end_bci: (\d+), handler_bci: (\d+), catch_type_index: Some\((\d+)\) \}',body,re.M)]
 profiles.append({'leg':leg,'name':name,'descriptor':descriptor,'class_sha256':sha(inp),'class_blake3':digest,'physical':physical,'nodes':nodes,'edges':edges,'physical_exception_table_rows':re.findall(r'^PHYSICAL_EXCEPTION_TABLE (.+)$',body,re.M),'physical_exception_semantics':[(o,a,b,h,catch_name(inp.read_bytes(),cp)) for o,a,b,h,cp in handlers]})
expected={'partialBreak','innerLoopBreak','innerSwitchBreak','terminalCase','caughtExceptionThenFallthrough'};assert seen=={(l,n) for l in ['javac8','javac23'] for n in expected}
for name in expected:
 a,b=[next(p for p in profiles if p['leg']==l and p['name']==name) for l in ['javac8','javac23']]
 for k in ['descriptor','physical','nodes','edges','physical_exception_semantics']:assert a[k]==b[k],(name,k)
assert not (root/'target').exists()
result={'schema':'conditional-boundary-public-ir-observations-acceptance-root-v1','status':'verified-observations-only','execution_sha256':sha(r/'execution.json'),'observer_sha256':e['observer_source_sha256'],'profiles':profiles,'product_acceptance':False,'cf12_complete':False,'scope':'trusted public reader/JVM IR facts on exact two original inputs; raw stage completion, physical decode, canonical identities/coverage and SSA block membership; no new conditional proof gate accepted'}
out=Path('/private/tmp/jarde-conditional-boundary-ir-acceptance-root-v2');out.mkdir(exist_ok=False);(out/'acceptance.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'profiles':len(profiles),'counts':[(p['leg'],p['name'],len(p['nodes']),len(p['edges'])) for p in profiles]}))
