from pathlib import Path
import hashlib,json,shutil,importlib.util,sys
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');P=Path('/private/tmp/jarde-cf11-testnestedloops-baseline-root-v1');A=R/'openspec/evidence/java-syntax-2026-10-12/cf11-testnestedloops-baseline-root-v1'
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert not A.exists();e=json.loads((P/'execution.json').read_text());c=json.loads((P/'checker-root-v5.json').read_text());b=json.loads(Path('/private/tmp/jarde-for-condition-final-lint-build-root-v3/execution.json').read_text());l=json.loads(Path('/private/tmp/jarde-cf11-testnestedloops-baseline-root-v1-launcher-receipt.json').read_text())
assert H(P/'execution.json')=='5635e60b5565a19ba4fab116d0b6ae73f72e62568e0761edeff2365291c95435'
assert e['status']=='observed' and len(e['commands'])==29 and e['source_tool_inputs_unchanged'] and e['source_tool_pins_before']==e['source_tool_pins_after']
assert c['status']=='verified_observation_capture' and c['actual_command_count']==29 and c['raw_stream_count']==58 and not c['source_map_gaps'] and not c['behavior_mismatches'] and c['all_method_physical_coverage_complete']
assert l['status']=='complete' and l['source_pins_before']==l['source_pins_after']==b['source_pins_before']==b['source_pins_after'] and len(l['source_pins_before'])==4718 and l['target_absent_after'] and not (R/'target').exists()
assert all(H(R/k)==v for k,v in l['source_pins_before'].items()) and e['cli']['sha256']==b['candidate_cli']['sha256']=='51c9a1326031fd10444f7e18a055c6daecef233423c90478257becf24d1e0bd4'
for row in e['commands']:
 assert row['exit_code']==0 and row['guard_stop'] is None and row['peak_target_bytes']==0 and row['free_bytes_after']>=5*1024**3
 for ref in row['streams'].values():
  q=Path(ref['path']);assert q.stat().st_size==ref['bytes'] and H(q)==ref['sha256']
by={r['label']:r for r in e['commands']};oracle=(P/'raw/2.stdout.raw').read_bytes();lines=oracle.decode().splitlines();assert len(lines)==8 and oracle.count(b'java.util.ConcurrentModificationException')==2 and oracle.count(b'java.lang.NullPointerException')==3
assert by['javac8-original-run']['streams']['stdout']['sha256']==by['javac23-original-run']['streams']['stdout']['sha256']==H(P/'raw/2.stdout.raw')
assert len(c['generated_compile_observations'])==8
for row in c['generated_compile_observations']:
 assert row['compile_exit']==row['runtime_exit']==0 and row['same_jdk_original'] and row['class_set_exact'] and row['stdout_sha256']==H(P/'raw/2.stdout.raw') and row['stderr_sha256']==hashlib.sha256(b'').hexdigest()
assert len(e['jarde_profiles'])==4
cp=Path('/private/tmp/jarde-for-condition-checker-root-v5/check_controls.py');assert H(cp)=='d338d6ae9b4bec67f9b2fa76db0f71dc04ea7d75c82f7155805bb8ba6975c51b';s=importlib.util.spec_from_file_location('nested_root_check',cp);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);parser=m.import_parser(e);methods=[]
for x in e['jarde_profiles']:
 text=Path(x['source_pin']['path']).read_text();assert text.count('while (')==2 and 'for (' not in text and text.count('ordinary_generic_source_unproved')==1
 for ref in [x['source_pin'],x['report_pin']]:assert H(ref['path'])==ref['sha256'] and Path(ref['path']).stat().st_size==ref['bytes']
 d=json.loads(Path(x['report_pin']['path']).read_text());ph=next(t for r in e['original_compile_rows'] if r['label']==x['jdk']+'-original' for t in r['classes'] if t['path'].endswith('/UpstreamNestedLoops.class'));jp=next(j for j in e['javap'] if j['jdk']==x['jdk']);rows=m.verify_class(parser,d,Path(ph['path']),Path(jp['stdout_pin']['path']).read_bytes(),'cf11/UpstreamNestedLoops');assert [len(r['instruction_bcis']) for r in rows]==[3,45] and all(r['physical_coverage_complete'] and r['quality']=='structured' for r in rows);methods.append({'label':x['label'],'methods':rows})
for profile in ['default','none']:
 t=(P/'jadx'/profile/'sources/cf11/UpstreamNestedLoops.java').read_text();assert t.count('for (String ')==2 and 'List<String> l1, List<String> l2' in t
result={'status':'accepted_fixed_baseline_behavior_and_source_gap_only','product_commit':'02ecd63ff33270cea4413793db2f99d6927dc65c','execution_sha256':H(P/'execution.json'),'checker_sha256':H(P/'checker-root-v5.json'),'root_verifier_sha256':H(__file__),'actual_commands':29,'actual_raw_streams':58,'source_pins':4718,'generated_full_class_runtime_legs':8,'original_runtime_legs':2,'original_oracle_lines':lines,'method_maps':methods,'presentation_gap':{'jarde':'two structured while loops and refused generic Signature/raw List parameters','jadx':'two String enhanced for loops and List<String> parameters','internal_first_rejection':'not traced; source review is static only'},'target_absent':True,'scope':'This fixed method and eight actual cases only; not CF11/71 completion or generic/foreach implementation acceptance.'}
A.mkdir(parents=True);shutil.copytree(P,A/'capture')
for name,src in [('frozen-inputs','/private/tmp/jarde-cf11-testnestedloops-inputs-root-v1'),('source-preparation-v2','/private/tmp/jarde-cf11-testnestedloops-input-luna-v2'),('source-preparation-v3','/private/tmp/jarde-cf11-testnestedloops-input-luna-v3'),('launcher-preparation-v1','/private/tmp/jarde-cf11-testnestedloops-baseline-launch-prep-luna-v1'),('source-review-luna-v1','/private/tmp/jarde-testnestedloops-source-review-luna-v1'),('launcher-invocation','/private/tmp/jarde-cf11-invocations-root-v1/testnestedloops-baseline-root-v1')]:shutil.copytree(src,A/name)
for q in [Path(__file__),Path('/private/tmp/jarde-cf11-testnestedloops-baseline-launch-root-v1.py'),Path('/private/tmp/jarde-cf11-testnestedloops-baseline-root-v1-launcher-receipt.json'),cp]:shutil.copyfile(q,A/q.name)
(A/'root-acceptance.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
for row in e['commands']:
 for stream,ref in row['streams'].items():assert H(A/'capture/raw'/f"{row['index']}.{stream}.raw")==ref['sha256']
files=[p for p in A.rglob('*') if p.is_file()];(A/'archive-inventory.json').write_text(json.dumps({'copied_files':[{'path':str(p.relative_to(A)),'bytes':p.stat().st_size,'sha256':H(p)} for p in sorted(files)],'all_58_raw_streams_checked':True},indent=2)+'\n')
print(json.dumps({'status':result['status'],'archive':str(A),'commands':29,'runtime_legs':8,'files':len(files)},ensure_ascii=False))
