#!/usr/bin/env python3
"""Independent read-only whole-class/source-map verifier; writes one exclusive result JSON."""
from __future__ import annotations
import argparse, ast, datetime, hashlib, json, os, re, stat, zipfile
from collections import Counter
from pathlib import Path
from blake3 import blake3
ROOT=Path('/Users/lordcasser/workspace/projects/jarde'); HERE=Path('/private/tmp/jarde-proved-if-join-control-replay-v1')
RESULTS=ROOT/'openspec/changes/preserve-proved-if-arm-join-origins/results'; OUT=RESULTS/'candidate-whole-classes-root-v1'; ACCEPT=RESULTS/'candidate-whole-classes-acceptance-root-v1.json'
FOR_RESULTS=ROOT/'openspec/changes/preserve-proved-for-latch-origins/results'
FOR_BUNDLE=FOR_RESULTS/'candidate-whole-classes-root-v1'
FOR_ACCEPTANCE=FOR_RESULTS/'candidate-whole-classes-acceptance-root-v1.json'
FOR_ACCEPTANCE_SHA='1f7a318a0c7da89744814414ec4a8cbdc22efbe6920ff601ede98f7b6c0b0866'
FOR_MANIFEST_SHA='38e80cb85429e0236f91b9828fafc145eebf23e990e79b4ef02e9609dc7c5525'
FOR_INVENTORY_SHA='4324037c2fdfa492c3a228c50bd08d77a03b78741c3e008eadb1d6e14453f5fd'
COLLECTOR=RESULTS/'prepare-candidate-root-v1.py'
CLI_PATH=Path('/private/tmp/jarde-proved-if-join-cli-v1'); METADATA_PATH=RESULTS/'candidate-cli-v1.json'
METADATA_SCHEMA='preserve-proved-if-arm-join-origins-candidate-cli-v1'; SOURCE_BASE=None
EVID=ROOT/'openspec/evidence/java-syntax-2026-10-10'
JDK_MANIFEST=ROOT/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'; JDK_MANIFEST_SHA='ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
JDK_TOOLS={}
GUARD_SOURCE=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'; GUARD_SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
BASES={'postfix':(EVID/'em23-variable-postfix-loop/baseline-root-v1','e8aec7964455ab84ffb12d984140e18bffe94e5cc0cf7f1eb5de06c21cc23a2d','89af5a986896541c5adb0dd8bed30c89d61038017f05d57634142e8416cce783',EVID/'em23-variable-postfix-loop/results/prepare-baseline-luna-v1.py','2e35326ebe7b2b60bd47b60419375088bc36142e053ea41f086f3cf534743089',EVID/'em23-variable-postfix-loop/results/independent-acceptance-luna-v3.json','946409416021061c868d671b929279937997efcf479da7c1bef7ab7ccabe49eb'), 'plain':(EVID/'one-arm-loop-controls/baseline-root-v1','6dfccb5ceeb854edf41b90e4136c23ce503699bb83bb17767444cd6fd6cfc18d','96d487211c0bcbb71b6fb06a19894fb3a8d0b59a44b5376f8a5a3a48aa7c1b96',EVID/'one-arm-loop-controls/prepare-baseline-luna-v1.py','a1ed035789f96f2a945bc9e8499bb4d1d089639082ccb59ce61933c97660879c',EVID/'one-arm-loop-controls/results/independent-acceptance-luna-v5.json','f95666277091007f178cd4a74170f22eddd7699a49b2e5db02c08fa27bbad450')}
METHODS={'postfix':[('<init>','()V'),('countEmpty','(Ljava/util/List;)I')],'plain':[('<init>','()V'),('prefixWhile','(ZI)I'),('noPrefix','(ZI)I'),('loopAndTail','(ZI)I'),('takenArm','(ZI)I')]}
PRODUCT_PATHS={"Cargo.toml","Cargo.lock","crates/jarde-jvm/Cargo.toml","crates/jarde-reader/Cargo.toml","crates/jarde-query/Cargo.toml","crates/jarde-java/Cargo.toml","crates/jarde-cli/Cargo.toml","crates/jarde-java/src/region.rs","crates/jarde-java/src/build.rs","crates/jarde-java/src/emit.rs","crates/jarde-java/src/report.rs","crates/jarde-java/src/lib.rs","src/class_source.rs","src/facade.rs","src/lib.rs","crates/jarde-cli/src/main.rs","crates/jarde-cli/src/task.rs"}
TEST_PATHS={".github/workflows/ci.yml","crates/jarde-java/tests/p3_loop_exit_gateways.rs","crates/jarde-java/tests/p3_loop_body_double_jumps.rs","crates/jarde-java/tests/p3_loop_terminal_return.rs","crates/jarde-java/tests/p3_effectful_exits.rs","tests/p3_loop_arm_join.rs","tests/p3_loop_boolean_exit.rs","tests/p5_corpus_fingerprint.rs"}
def sha(b):return hashlib.sha256(b).hexdigest()
def package_of_text(text):
 m=re.search(r'(?m)^\s*package\s+([\w.]+)\s*;',text);return m.group(1) if m else None
def sha_file(p):return sha(p.read_bytes())
def reject(m):raise RuntimeError(m)
def expected_pins():
 inc=re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"\s*,?\s*\)'); canonical=set()
 for rel in sorted(TEST_PATHS):
  if rel.endswith('.rs'):
   for x in inc.findall((ROOT/rel).read_text()):canonical.add((ROOT/rel).parent.joinpath(x).resolve().relative_to(ROOT).as_posix())
 groups={'candidate_sources':PRODUCT_PATHS,'test_sources':TEST_PATHS,'canonical_files':canonical}
 return {k:{p:sha_file(ROOT/p) for p in sorted(v)} for k,v in groups.items()},groups
def helpers(name):
 base,msha,isha,src,csha,accept,asha=BASES[name]
 if sha_file(src)!=csha:reject(name+' baseline collector pin')
 wanted={'file_record','inventory_rows','package_of','package_of_text','copy_runner','adapt_runner','class_files','compile_run','parse_javap','parse_javap_methods','method_key','verify_report_map','b3','sha'}
 tree=ast.parse(src.read_bytes()); nodes=[n for n in tree.body if isinstance(n,(ast.FunctionDef,ast.ClassDef)) and n.name in wanted]; imports=[n for n in tree.body if isinstance(n,ast.ImportFrom) and n.module=='blake3']
 ns={'__name__':'_verify_pinned_helpers','__file__':str(src),'Path':Path,'re':re,'hashlib':hashlib,'blake3':blake3,'OUT':OUT,'ROOT':ROOT,'CLASS_NAME':'VariablePostfixLoop' if name=='postfix' else 'PlainOneArmLoops'}
 exec(compile(ast.fix_missing_locations(ast.Module(body=imports+nodes,type_ignores=[])),str(src),'exec'),ns)
 ns['METHOD_KEYS']=METHODS[name]
 if name=='postfix':ns['METHODS']=set(METHODS[name])
 else:ns['METHOD_FLAGS']={'<init>()V':1,'prefixWhile(ZI)I':9,'noPrefix(ZI)I':9,'loopAndTail(ZI)I':9,'takenArm(ZI)I':9}
 return ns
def closed_base(name):
 base,msha,isha,_,_,accept,asha=BASES[name]; mb=(base/'manifest.json').read_bytes(); ib=(base/'file-inventory.json').read_bytes()
 if sha(mb)!=msha or sha(ib)!=isha:reject(name+' baseline manifest/inventory digest')
 if sha(accept.read_bytes())!=asha or json.loads(accept.read_bytes()).get('verified') is not True:reject(name+' baseline independent acceptance pin')
 inv=json.loads(ib); actual={p.relative_to(base).as_posix():p for p in base.rglob('*') if p.is_file() and p!=base/'file-inventory.json'}
 if set(actual)!={r['path'] for r in inv}:reject(name+' baseline closure')
 for r in inv:
  b=actual[r['path']].read_bytes()
  if len(b)!=r['bytes'] or sha(b)!=r['sha256']:reject(name+' baseline member '+r['path'])

def closed_for_bundle_baseline():
 raw=FOR_ACCEPTANCE.read_bytes()
 if sha(raw)!=FOR_ACCEPTANCE_SHA:reject('accepted For bundle acceptance SHA mismatch')
 accepted=json.loads(raw)
 if accepted.get('schema')!='preserve-proved-for-latch-origins-candidate-acceptance-root-v1' or accepted.get('verified') is not True or accepted.get('full_bci_acceptance') is not True or accepted.get('status')!='accepted':reject('previous For bundle is not fully accepted')
 manifest_path=FOR_BUNDLE/'manifest.json'; inventory_path=FOR_BUNDLE/'file-inventory.json'
 manifest_raw=manifest_path.read_bytes(); inventory_raw=inventory_path.read_bytes()
 if sha(manifest_raw)!=FOR_MANIFEST_SHA or accepted.get('manifest_sha256')!=FOR_MANIFEST_SHA:reject('accepted For manifest SHA mismatch')
 if sha(inventory_raw)!=FOR_INVENTORY_SHA:reject('accepted For inventory SHA mismatch')
 inventory=json.loads(inventory_raw); actual={p.relative_to(FOR_BUNDLE).as_posix():p for p in FOR_BUNDLE.rglob('*') if p.is_file() and p!=inventory_path}
 if set(actual)!={row['path'] for row in inventory}:reject('accepted For bundle inventory is not closed')
 for row in inventory:
  raw=actual[row['path']].read_bytes()
  if len(raw)!=row['bytes'] or sha(raw)!=row['sha256']:reject('accepted For bundle member mismatch '+row['path'])
 manifest=json.loads(manifest_raw)
 if manifest.get('schema')!='preserve-proved-for-latch-origins-candidate-observation-v1' or manifest.get('status')!='observations-recorded' or manifest.get('failures'):reject('accepted For observation manifest is not clean')
 return {'acceptance_path':str(FOR_ACCEPTANCE),'acceptance_sha256':FOR_ACCEPTANCE_SHA,'manifest_path':str(manifest_path),'manifest_sha256':FOR_MANIFEST_SHA,'inventory_path':str(inventory_path),'inventory_sha256':FOR_INVENTORY_SHA,'files':len(inventory),'full_bci_acceptance':True},manifest

def source_origin_keys(doc, helper):
 keys=Counter()
 for method in doc.get('methods',[]):
  key=helper['method_key'](method['item'])
  source_map=method.get('outcome',{}).get('report',{}).get('source_map') or {}
  for segment in source_map.get('segments',[]):
   origin=segment.get('origin') or {}
   parts=([origin['primary']] if origin.get('primary') else [])+origin.get('derived',[])
   for part in parts:
    keys[(key,json.dumps(part,sort_keys=True,separators=(',',':')))]+=1
 return keys

def verify_old_origins_against_for_bundle(current_manifest, for_manifest, helpers_by_suite):
 checks={}
 for suite in ('postfix','plain'):
  old={(r['leg'],r['mode']):r for r in for_manifest['suites'][suite]['renders'] if r.get('kind')=='jarde'}
  new={(r['leg'],r['mode']):r for r in current_manifest['suites'][suite]['renders'] if r.get('kind')=='jarde'}
  expected={(leg,mode) for leg in ('javac8','javac23') for mode in ('default','all')}
  if set(old)!=expected or set(new)!=expected:reject(suite+' accepted-For baseline/render matrix mismatch')
  for pair in sorted(expected):
   old_doc=json.loads((FOR_BUNDLE/old[pair]['document']).read_bytes())
   new_doc=json.loads((OUT/new[pair]['document']).read_bytes())
   missing=source_origin_keys(old_doc,helpers_by_suite[suite])-source_origin_keys(new_doc,helpers_by_suite[suite])
   if missing:reject(f'{suite}/{pair} dropped prior source origins: {list(missing.items())[:3]}')
  checks[suite]={'legs':2,'profiles':2,'all_prior_origin_owners_and_provenance_preserved':True}
 return checks

def verify_postfix_against_for_baseline(current_manifest, for_manifest):
 old={ (r['leg'],r['mode']):r for r in for_manifest['suites']['postfix']['renders'] if r.get('kind')=='jarde' }
 new={ (r['leg'],r['mode']):r for r in current_manifest['suites']['postfix']['renders'] if r.get('kind')=='jarde' }
 expected={(leg,mode) for leg in ('javac8','javac23') for mode in ('default','all')}
 if set(old)!=expected or set(new)!=expected:reject('postfix For baseline/render matrix mismatch')
 for key in sorted(expected):
  old_doc=json.loads((FOR_BUNDLE/old[key]['document']).read_bytes())
  new_doc=json.loads((OUT/new[key]['document']).read_bytes())
  if old_doc.get('text')!=new_doc.get('text'):reject(f'postfix generated body changed from accepted For baseline: {key}')
  def maps(doc):return {h['method_key'](row['item']):row['outcome']['report'].get('source_map') for row in doc['methods']}
  if maps(old_doc)!=maps(new_doc):reject(f'postfix source spans/origins changed from accepted For baseline: {key}')
 return {'legs':2,'profiles':2,'body_and_source_maps_identical_to_accepted_for_bundle':True,
         'method':'countEmpty(Ljava/util/List;)I','preserves_prior_For_body_and_origins_including_BCI20':True}

def stream(row,kind):
 r=row['streams'][kind]; p=OUT/r['path']; b=p.read_bytes()
 if len(b)!=r['bytes'] or sha(b)!=r['sha256']:reject('raw command stream mismatch '+r['path'])
 return b
def cmd_ok(row):
 if row.get('cwd')!=str(ROOT):reject('command cwd mismatch '+row.get('label','?'))
 if row.get('exit_code')!=0 or row.get('guard_stop') is not None or row.get('free_bytes_after',0)<5*1024**3 or row.get('peak_target_bytes',2*1024**3)>1024**3:reject('command failed/guard mismatch '+row.get('label','?'))
 stream(row,'stdout');stream(row,'stderr')
def runtime_raw(command,commands):
 r=commands.get(command['label'])
 if not r:reject('runtime command missing')
 cmd_ok(r);return r['exit_code'],stream(r,'stdout'),stream(r,'stderr')
def run_case(case,commands,oracle=None):
 row=case['row']; comp=row.get('compile'); runtime=row.get('runtime')
 if not row.get('compile_success') or not row.get('class_set_exact') or row.get('actual_class_paths')!=row.get('expected_class_paths'):reject(case['kind']+' compile/class-set failure '+case.get('leg',''))
 if not comp:reject('compile argv record missing')
 cmd_ok(comp)
 av=comp['argv'];
 leg=case.get('leg')
 if leg not in JDK_TOOLS or av[0]!=JDK_TOOLS[leg]['javac']:reject('compile did not use pinned javac for '+str(leg))
 if '-source' not in av or av[av.index('-source')+1]!='8' or '-target' not in av or av[av.index('-target')+1]!='8' or '-classpath' not in av or '-sourcepath' not in av:reject('compile options do not isolate Java 8 class/source paths')
 cp=Path(av[av.index('-classpath')+1]); sp=Path(av[av.index('-sourcepath')+1])
 if cp!=sp or not cp.is_dir() or any(cp.iterdir()) or str(cp).find(str(OUT))!=0:reject('compile classpath/sourcepath is not the same fresh empty directory')
 if '-d' not in av or not Path(av[av.index('-d')+1]).is_dir() or Path(av[av.index('-d')+1])!=Path(row['class_output']):reject('fresh class output directory missing')
 source_args=[str(OUT/z['path']) for z in row['source_files']]
 if av[-len(source_args):]!=source_args:reject('compile argv source list differs from recorded files')
 if not row.get('runtime_success') or not runtime:reject(case['kind']+' runtime absent/failing '+case.get('leg',''))
 cmd_ok(runtime)
 if '-Xverify:all' not in runtime['argv'] or runtime['argv'][0]!=JDK_TOOLS[leg]['java'] or runtime['argv'][-1]!='Runner' and not runtime['argv'][-1].endswith('.Runner'):reject('runtime missing -Xverify:all, pinned java, or Runner entry point')
 if '-cp' not in runtime['argv'] or Path(runtime['argv'][runtime['argv'].index('-cp')+1])!=Path(row['class_output']):reject('runtime did not use fresh class output only')
 actual=(runtime['exit_code'],stream(runtime,'stdout'),stream(runtime,'stderr'))
 if oracle is not None and actual!=oracle:reject(case['kind']+' raw runtime differs from same-JDK original')
 return actual
def main():
 global SOURCE_BASE
 ap=argparse.ArgumentParser();ap.add_argument('--source-base',required=True);ap.add_argument('--manifest',default=str(OUT/'manifest.json'));ap.add_argument('--build',required=True);a=ap.parse_args()
 SOURCE_BASE=a.source_base
 if len(SOURCE_BASE)!=40 or any(c not in '0123456789abcdef' for c in SOURCE_BASE):reject('--source-base must be lowercase 40-hex')
 if ACCEPT.exists():raise SystemExit('refusing to overwrite acceptance JSON')
 for n in BASES:closed_base(n)
 hs={n:helpers(n) for n in BASES}; expected,sets=expected_pins()
 manifest_path=Path(a.manifest)
 if not manifest_path.is_file():reject('candidate manifest missing')
 m=json.loads(manifest_path.read_bytes()); for_baseline,for_manifest=closed_for_bundle_baseline();
 if m.get('accepted_for_bundle_baseline')!=for_baseline:reject('collector did not bind the accepted For bundle baseline')
 cli=m['cli']; meta=m['metadata']; cp=Path(cli['path']).resolve();mp=Path(meta['path']).resolve()
 build=Path(a.build); build_resolved=build.resolve()
 if build.name!='execution.json' or not build_resolved.is_relative_to(RESULTS.resolve()):reject('--build must be execution.json under this change results directory')
 if meta.get('build_result_path')!=str(build_resolved):reject('manifest does not bind exact --build path')
 if m.get('collector',{}).get('path')!=str(COLLECTOR) or m.get('collector',{}).get('sha256')!=sha_file(COLLECTOR):reject('collector path/SHA mismatch')
 if sha_file(cp)!=cli['sha256'] or stat.S_IMODE(cp.stat().st_mode)!=0o555 or cli.get('mode')!='0o555':reject('CLI bytes/mode changed')
 if sha_file(mp)!=meta['sha256']:reject('metadata bytes changed')
 md=json.loads(mp.read_bytes()); eb=build_resolved.read_bytes(); execution=json.loads(eb)
 if sha(eb)!=md.get('build_result_sha256') or sha(eb)!=meta.get('build_result_sha256'):reject('build result SHA binding')
 if cp.resolve()!=CLI_PATH or mp.resolve()!=METADATA_PATH.resolve() or md.get('schema')!=METADATA_SCHEMA or md.get('metadata_path')!=str(mp) or md.get('cli_path')!=str(cp) or md.get('cli_sha256')!=cli['sha256'] or md.get('uncommitted_if_arm_join_product') is not True or md.get('source_commit_base')!=SOURCE_BASE or meta.get('schema')!=md.get('schema') or meta.get('source_commit_base')!=md.get('source_commit_base') or meta.get('uncommitted_if_arm_join_product') is not True:reject('CLI/metadata path/schema/source-base binding')
 if m['metadata'].get('source_pins')!=expected or md.get('candidate_sources')!=expected['candidate_sources'] or md.get('test_sources')!=expected['test_sources'] or md.get('canonical_files')!=expected['canonical_files']:reject('product/test/canonical pins mismatch')
 if m['metadata'].get('source_pins_before')!=expected or m['metadata'].get('source_pins_after')!=expected or m['metadata'].get('source_pins_after_replay')!=expected:reject('validation/replay source pins before/after mismatch')
 schema=execution.get('schema',''); version_match=re.fullmatch(r'preserve-proved-if-arm-join-origins-validation-build-root-v([1-9][0-9]*)',schema)
 if not version_match:reject('build execution schema is outside this change validation-build family')
 version=version_match.group(1)
 if build_resolved.parent.name!=f'validation-build-root-v{version}' or build_resolved.parent.parent!=RESULTS.resolve():reject('--build directory does not match its validation schema version')
 runner=(build_resolved.parent.parent/f'run-validation-build-root-v{version}.py').resolve(); runner_row=execution.get('validation_runner',{})
 if runner_row.get('path')!=str(runner) or not runner.is_file() or runner_row.get('sha256')!=sha_file(runner):reject('validation runner path/SHA does not match build schema and live file')
 template=execution.get('guarded_runner_template',{})
 if template.get('path')!=str(GUARD_SOURCE) or template.get('sha256')!=GUARD_SHA or sha_file(GUARD_SOURCE)!=GUARD_SHA:reject('validation guarded-runner template pin mismatch')
 freeze=execution.get('freeze',{})
 if execution.get('status')!='validation-passed-cli-frozen' or execution.get('preflight',{}).get('source_pins_before')!=expected or execution.get('preflight',{}).get('source_pins_after')!=expected or freeze.get('cli_path')!=str(cp) or freeze.get('cli_sha256')!=cli['sha256'] or freeze.get('cli_mode')!='0o555' or freeze.get('metadata_path')!=str(mp) or freeze.get('source_commit_base')!=md.get('source_commit_base') or freeze.get('source_commit_base')!=SOURCE_BASE or freeze.get('uncommitted_if_arm_join_product') is not True or execution.get('uncommitted_if_arm_join_product') is not True:reject('validation execution freeze/pins mismatch')
 # Rebuild the exact include path set from the frozen root runner inputs.
 if set(expected['candidate_sources'])!=PRODUCT_PATHS or set(expected['test_sources'])!=TEST_PATHS or len(expected['candidate_sources'])!=17 or len(expected['test_sources'])!=8:reject('literal product/test path set mismatch')
 rows=json.loads((OUT/'file-inventory.json').read_bytes()); actual={p.relative_to(OUT).as_posix():p for p in OUT.rglob('*') if p.is_file() and p.name!='file-inventory.json'}
 if set(actual)!={r['path'] for r in rows}:reject('candidate output inventory not closed')
 for r in rows:
  b=actual[r['path']].read_bytes()
  if len(b)!=r['bytes'] or sha(b)!=r['sha256']:reject('candidate output member hash '+r['path'])
 m=json.loads(manifest_path.read_bytes()); commands={c['label']:c for c in m['commands']}
 if len(commands)!=len(m['commands']):reject('duplicate command labels')
 if len(commands)!=57:reject('whole-class replay command count is not exactly 57')
 for c in commands.values():cmd_ok(c)
 if m.get('schema')!='preserve-proved-if-arm-join-origins-candidate-observation-v1' or m.get('status')!='observations-recorded':reject('candidate manifest status/schema')
 if m['failures']:reject('collector recorded one or more failed observations')
 if sha_file(GUARD_SOURCE)!=GUARD_SHA:reject('guarded runner pin changed')
 if sha_file(JDK_MANIFEST)!=JDK_MANIFEST_SHA:reject('fixed JDK manifest changed')
 jdkdoc=json.loads(JDK_MANIFEST.read_bytes()); jlegs={x['leg']:x for x in jdkdoc['legs']}
 global JDK_TOOLS
 JDK_TOOLS={leg:{tool:fact['path'] for tool,fact in row['jdk_tools'].items()} for leg,row in jlegs.items()}
 if set(m['jdk']['legs'])!={'javac8','javac23'}:reject('dual JDK tool matrix incomplete')
 for leg,toolrows in m['jdk']['legs'].items():
  if set(toolrows)!={'java','javac','javap'}:reject('JDK tool set mismatch '+leg)
  if leg not in jlegs:reject('unexpected JDK leg')
  for tool,r in toolrows.items():
   frozen=jlegs[leg].get('jdk_tools',{}).get(tool,{})
   if not r['ok'] or r['path']!=frozen.get('path') or r['expected_sha256']!=frozen.get('sha256') or not Path(r['path']).is_file() or sha_file(Path(r['path']))!=r['expected_sha256']:reject('JDK tool pin mismatch '+leg+'/'+tool)
 jadx=m['jadx']; jp=Path(jadx['path'])
 if str(jp)!='/opt/homebrew/bin/jadx' or not jp.is_file() or sha_file(jp)!=jadx['sha256'] or jadx.get('version_observed')!='1.5.6':reject('JADX fixed launcher/version mismatch')
 vcmd=jadx['version_command']
 if vcmd['argv']!=[str(jp),'--version'] or stream(vcmd,'stdout').strip()!=b'1.5.6':reject('JADX version command/raw mismatch')
 if len(m['suites'])!=2:reject('suite inventory mismatch')
 for suite,row in m['fixtures'].items():
  if suite not in BASES:reject('unknown fixture '+suite)
  base=BASES[suite][0]; src=base/'original-sources'/('VariablePostfixLoop.java' if suite=='postfix' else 'PlainOneArmLoops.java'); runner=base/'original-sources/Runner.java'
  if row['source_path']!=str(src) or row['runner_path']!=str(runner) or sha_file(src)!=row['source_sha256'] or sha_file(runner)!=row['runner_sha256']:reject('Runner/source template pin mismatch '+suite)
 accepted_maps={}
 for suite in ('postfix','plain'):
  s=m['suites'][suite]; h=hs[suite]; keys=set(METHODS[suite]); originals={x['leg']:x for x in s['cases'] if x['kind']=='original'}; jadxs={(x['leg'],x['profile']):x for x in s['cases'] if x['kind']=='jadx'}
  if set(originals)!={'javac8','javac23'} or set(jadxs)!={(l,p) for l in ('javac8','javac23') for p in ('default','none')}:reject(suite+' input case matrix')
  renders={(x['leg'],x['mode']):x for x in s['renders'] if x['kind']=='jarde'}
  if set(renders)!={(l,p) for l in ('javac8','javac23') for p in ('default','all')}:reject(suite+' candidate render matrix')
  oracle={leg:run_case(row,commands) for leg,row in originals.items()}
  for leg,entry in originals.items():
   case=entry['row']; source_row=next(x for x in case['source_files'] if x['path'].endswith('\\'+('VariablePostfixLoop.java' if suite=='postfix' else 'PlainOneArmLoops.java')) or x['path'].endswith('/'+('VariablePostfixLoop.java' if suite=='postfix' else 'PlainOneArmLoops.java')))
   runner_row=next(x for x in case['source_files'] if x['path'].endswith('/Runner.java'))
   source_bytes=(BASES[suite][0]/'original-sources'/('VariablePostfixLoop.java' if suite=='postfix' else 'PlainOneArmLoops.java')).read_bytes()
   runner_bytes=(BASES[suite][0]/'original-sources/Runner.java').read_bytes()
   if (OUT/source_row['path']).read_bytes()!=source_bytes or (OUT/runner_row['path']).read_bytes()!=runner_bytes:reject(suite+'/'+leg+' original inputs differ from frozen source/Runner')
  for (leg,profile),case in jadxs.items():
   run_case(case,commands,oracle[leg])
   jc=next(x for x in s['renders'] if x.get('kind')=='jadx' and x.get('profile')==profile)['command']
   expected_jadx=[str(jp),'--no-res','--config','none','--threads-count','1']+(['--rename-flags','none'] if profile=='none' else [])+['-d',str(OUT/'cases'/suite/'jadx'/profile),str(OUT/'cases'/suite/'input.jar')]
   if jc['argv']!=expected_jadx:reject(suite+'/'+profile+' JADX argv mismatch')
   generated=OUT/case['row']['source_files'][0]['path']; runner=OUT/case['row']['source_files'][1]['path']
   if generated.name!=('VariablePostfixLoop.java' if suite=='postfix' else 'PlainOneArmLoops.java'):reject(suite+'/'+leg+'/'+profile+' JADX class source filename')
   pkg=package_of_text(generated.read_text(encoding='utf-8')); rr=BASES[suite][0]/'original-sources/Runner.java'; exp=(f'package {pkg};\n\n' if pkg else '')+rr.read_text(encoding='utf-8')
   if runner.read_text(encoding='utf-8')!=exp:reject(suite+'/'+leg+'/'+profile+' JADX Runner adaptation')
  jar=OUT/'cases'/suite/'input.jar'
  with zipfile.ZipFile(jar) as z:
   if z.namelist()!=[('VariablePostfixLoop' if suite=='postfix' else 'PlainOneArmLoops')+'.class'] or z.read(z.namelist()[0])!=(OUT/next(f['path'] for f in originals['javac23']['row']['classes'] if f['path'].endswith(('VariablePostfixLoop' if suite=='postfix' else 'PlainOneArmLoops')+'.class'))).read_bytes():reject(suite+' JADX jar is not the exact original class')
  for leg in ('javac8','javac23'):
   original_row=originals[leg]['row']; original_file=next(OUT/f['path'] for f in original_row['classes'] if f['path'].endswith(h['CLASS_NAME']+'.class'))
   jcmd=original_row['javap']['command']
   if jcmd['argv']!=[m['jdk']['legs'][leg]['javap']['path'],'-p','-c','-s','-v',str(original_file)]:reject(suite+'/'+leg+' javap argv mismatch')
   original_bytes=original_file.read_bytes(); javap_path=OUT/original_row['javap']['path']; javap_text=javap_path.read_text(); parser=h.get('parse_javap',h.get('parse_javap_methods')); physical=parser(javap_text)
   ds={}
   for mode in ('default','all'):
    r=renders[(leg,mode)]; doc_path=OUT/r['document']; doc_bytes=doc_path.read_bytes()
    if sha(doc_bytes)!=r['document_sha256'] or r['exit']!=0:reject(suite+'/'+leg+'/'+mode+' CLI render')
    doc=json.loads(doc_bytes);ds[mode]=doc
    expected_argv=[cli['path'],'class-source','--input',str(original_file),'--class',('VariablePostfixLoop' if suite=='postfix' else 'PlainOneArmLoops'),'--policy','single-class','--release','8','--format','json']+(['--evidence','all'] if mode=='all' else [])
    if r['command']['argv']!=expected_argv:reject(suite+'/'+leg+'/'+mode+' CLI argv differs from frozen protocol')
    if r.get('default_all_text_equal') is not True or r.get('default_all_source_maps_equal') is not True:reject(suite+'/'+leg+' default/all equality')
    generated=OUT/r['generated_source']
    if generated.read_text(encoding='utf-8')!=doc.get('text'):reject(suite+'/'+leg+'/'+mode+' generated source differs from JSON text')
    # Recompute report-map schema using the pinned helper; inspect its actual per-method schema.
    facts=h['verify_report_map'](doc,h['b3'](original_bytes),original_bytes,physical,True) if suite=='postfix' else h['verify_report_map'](doc,original_bytes,physical,mode)
    fact_rows=facts['methods'] if suite=='postfix' else facts['method_presentation_facts']
    if {tuple(x['method']) if isinstance(x['method'],list) else tuple(x['method']) for x in fact_rows}!=keys:reject(suite+'/'+leg+'/'+mode+' map method set')
    for fact in fact_rows:
     if fact.get('source_map_state')!='complete':reject(suite+'/'+leg+'/'+mode+' source-map state incomplete: '+str(fact['method']))
     if suite=='postfix':complete=fact['bci_coverage_complete']; origins=fact['all_present_origins_bind_to_exact_method_and_javap']
     else:
      complete=fact['bci_coverage_complete']; origins=fact['origin_bindings_and_bcis_valid']
      if fact['outcome_kind']!='recovered' or fact['quality']!='structured' or fact['representation']!='java' or fact['content']!='contains_statements' or fact['fallbacks']!=[] or fact['validation_errors']:reject(suite+'/'+leg+'/'+mode+' method presentation failed: '+str(fact['method']))
     if not complete or not origins:reject(suite+'/'+leg+'/'+mode+' incomplete physical BCI/source owner: '+str(fact['method']))
    cr=r['complete_class_compile_run']
    case={'kind':'jarde','leg':leg,'row':cr}
    actual=run_case(case,commands,oracle[leg])
    if not r['runtime_raw_match'] or actual!=oracle[leg]:reject(suite+'/'+leg+'/'+mode+' complete-class raw mismatch')
    # Runner adaptation must be exactly package insertion into frozen Runner bytes.
    runner=OUT/r['runner_adaptation']; original_runner=(BASES[suite][0]/'original-sources/Runner.java').read_text(encoding='utf-8')
    pkg=package_of_text(doc['text']); expected_runner=(f'package {pkg};\n\n' if pkg else '')+original_runner
    if runner.read_text(encoding='utf-8')!=expected_runner:reject(suite+'/'+leg+'/'+mode+' Runner changed beyond package adaptation')
    srcs=[OUT/x['path'] for x in cr['source_files']]
    if set(srcs)!={generated,runner}:reject(suite+'/'+leg+'/'+mode+' compile source inventory differs')
    argv=cr['compile']['argv']
    if argv[-2:]!=[str(generated),str(runner)]:reject(suite+'/'+leg+'/'+mode+' compile argv source list differs')
    expected_runner_path=BASES[suite][0]/'original-sources/Runner.java'
    original_input=BASES[suite][0]/'original-sources'/('VariablePostfixLoop.java' if suite=='postfix' else 'PlainOneArmLoops.java')
    if not original_input.is_file() or sha_file(original_input)!=( '17147219c9e524d18d62063f932b1ebb3ae976cb6f3bd3fddd2bfd8a58889199' if suite=='postfix' else '8f5ccc668017113caf93002d3e07853206412ca581a23dfb98950ad7393805b4') or not expected_runner_path.is_file():reject(suite+' frozen fixture pin mismatch')
   def methodmaps(doc):return {h['method_key'](x['item']):x['outcome']['report'].get('source_map') for x in doc['methods']}
   if ds['default'].get('text')!=ds['all'].get('text') or methodmaps(ds['default'])!=methodmaps(ds['all']):reject(suite+'/'+leg+' profile output/source-map mismatch')
   accepted_maps[suite+'/'+leg]={'method_count':len(keys),'default_all_equal':True,'all_physical_bcis_complete':True}
 old_origin_preservation=verify_old_origins_against_for_bundle(m,for_manifest,hs)
 postfix_for_baseline=verify_postfix_against_for_baseline(m,for_manifest)
 result={'schema':'preserve-proved-if-arm-join-origins-candidate-acceptance-root-v1','verified':True,'full_bci_acceptance':True,'status':'accepted','manifest_path':str(manifest_path),'manifest_sha256':sha(manifest_path.read_bytes()),'candidate_cli':cli,'candidate_metadata':meta,'build_result_sha256':sha(eb),'build_result_path':str(build_resolved),'baseline_helper_schema_rechecks':accepted_maps,'accepted_for_bundle_baseline':for_baseline,'old_origin_preservation':old_origin_preservation,'postfix_for_baseline_comparison':postfix_for_baseline,'commands':len(commands),'generated_whole_classes':8,'original_whole_classes':4,'jadx_whole_classes':8,'source_commit_base':SOURCE_BASE,'created_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
 if ACCEPT.exists():reject('acceptance path already exists')
 ACCEPT.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 print('ACCEPT: independent full-class/raw/source-map gates passed')
if __name__=='__main__':main()
