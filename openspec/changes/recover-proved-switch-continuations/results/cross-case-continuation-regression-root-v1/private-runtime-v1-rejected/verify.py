#!/usr/bin/env python3
"""Independent acceptance of the controlled full-class CandidateContinuation replay."""
from __future__ import annotations
import argparse, hashlib, json, re, stat, sys
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde').resolve()
BUILD_EVIDENCE=Path('/private/tmp/jarde-compound-final-replay-verifier-luna-v2/build_evidence.py')
BUILD_EVIDENCE_SHA='8b06e82de2fc696b3b82d32b444026b57a2d28f8c5cc51fef87d4ffcbe4ffec3'
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
JDK_MANIFEST=ROOT/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA='ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
JADX=Path('/opt/homebrew/bin/jadx').resolve()
JADX_SHA='64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
FIXTURE=ROOT/'openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/two-member-control/cross-case/CandidateContinuation.class'
FIXTURE_SHA='9aa523e64295cdf5738d1240ea2ba4b30b136cfec4a5b90cd362a61bfdb6c238'
ORACLE=ROOT/'openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/fixtures/cross-case/Join.class'
ORACLE_SHA='bec9b2bbbbb1b0ec2af20f125368bfd66b855efa4d188bac3d735d99cbfcf6c0'
JOIN_RAW=Path('/private/tmp/jarde-cross-case-continuation-diagnosis-root-v1/1.stdout.raw')
JOIN_RAW_SHA='c58ae7e7ab27ac53d2826e17c9e36a8c16f567ccb9c3790c25046cc831f1132a'
JAVAP=ROOT/'openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/two-member-control/1.stdout.raw'
JAVAP_SHA='37f039fe831d63981da1bf9dee369582a86d87e0653001eb5803a5046545407a'
GENERATION=ROOT/'openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/two-member-control/generation.json'
GENERATION_SHA='f956f5b2649201a1f61d1e49806c6afbf653868c3cbccb000316cfc1d9793a5f'
EXPECTED=b'MIN=30\n-3=30\n-1=30\n0=30\n1=30\n2=20\n3=30\n4=30\nMAX=30\n'
TARGET_BCIS={0,1,2,3,28,30,31,34,36,37,39,40,41}
LEGS=tuple((jdk,profile) for jdk in ('javac8','javac23') for profile in ('original','default','all','jadx'))
LABELS=['class-source-default','class-source-all','jadx-full-class']+[f'{j}-{p}-{phase}' for j,p in LEGS for phase in ('compile','runtime')]

def require(ok,msg):
 if not ok: raise AssertionError(msg)
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def pin_matches(entry):
 p=Path(entry['path'])
 return p.is_file() and p.stat().st_size==entry['bytes'] and sha(p)==entry['sha256']
def byte_text(value):
 if isinstance(value,str): return value
 return bytes(value).decode('utf-8')
def raw_for(out,row,index):
 require(row.get('index')==index,f'command index drift {index}')
 got={}
 for key in ('stdout','stderr'):
  e=row.get('streams',{}).get(key); require(isinstance(e,dict) and pin_matches(e),f'raw stream pin {index}/{key}')
  p=Path(e['path']).resolve(); require(p==(out/'guard'/f'{index}.{key}.raw').resolve(),f'raw stream location {index}/{key}')
  got[key]=p.read_bytes()
 return got
def parse_cp(text):
 pool={}
 in_pool=False
 for line in text.splitlines():
  if line.strip()=='Constant pool:': in_pool=True; continue
  if in_pool and line.strip()=='{': break
  if not in_pool: continue
  m=re.match(r's*#(d+) = (S+)(?:s+(.*))?$',line)
  if not m: continue
  idx,tag,body=int(m.group(1)),m.group(2),m.group(3) or ''
  refs=[int(x) for x in re.findall(r'#(d+)',body)]
  utf8=None
  u=re.search(r'= Utf8s+(.*)$',line)
  if u: utf8=u.group(1)
  pool[idx]={'tag':tag,'refs':refs,'utf8':utf8}
 require(bool(pool),'javap constant pool is absent')
 for idx,row in pool.items():
  for ref in row['refs']: require(ref in pool,f'constant pool #{idx} references absent #{ref}')
  if row['tag']=='Class':
   require(len(row['refs'])==1 and pool[row['refs'][0]]['tag']=='Utf8',f'class #{idx} lacks UTF8 closure')
  elif row['tag']=='NameAndType':
   require(len(row['refs'])==2 and all(pool[x]['tag']=='Utf8' for x in row['refs']),f'name/type #{idx} lacks UTF8 closure')
  elif row['tag']=='Methodref':
   require(len(row['refs'])==2 and pool[row['refs'][0]]['tag']=='Class' and pool[row['refs'][1]]['tag']=='NameAndType',f'methodref #{idx} is not closed')
  else: require(row['tag']=='Utf8',f'unexpected constant-pool tag {row["tag"]}')
 return pool
def map_origins(report,fixture_digest,fixture_len,method_name,descriptor):
 sm=report.get('source_map'); require(isinstance(sm,dict) and isinstance(sm.get('segments'),list) and sm['segments'],'source map missing')
 bcis=set(); text=report.get('text','')
 for seg in sm['segments']:
  a,b=seg.get('start'),seg.get('end')
  require(isinstance(a,int) and isinstance(b,int) and 0<=a<=b<=len(text),'source-map span outside complete method text')
  origin=seg.get('origin',{}); origins=[]
  if isinstance(origin.get('primary'),dict): origins.append(origin['primary'])
  origins.extend(x for x in origin.get('derived',[]) if isinstance(x,dict))
  require(origins,'source-map segment has no physical owner')
  for o in origins:
   m=o.get('method',{}); owner=m.get('owner',{}).get('class_bytes',{})
   require(owner.get('digest')==fixture_digest and owner.get('length')==fixture_len,'source-map owner differs from candidate class')
   require(byte_text(m.get('name',[]))==method_name and byte_text(m.get('descriptor',[]))==descriptor,'source-map method owner mismatch')
   bci=o.get('bci'); require(isinstance(bci,int),'source-map BCI missing'); bcis.add(bci)
   cp=o.get('cp')
   require(cp is None or cp in CP_POOL,'source-map constant-pool link is unresolved')
 return bcis
def api_methods(api,label,digest,length):
 require(api.get('outcome')=='performed',f'{label}: whole-class API not performed')
 cls=api.get('class',{}).get('class_bytes',{})
 require(cls.get('digest')==digest and cls.get('length')==length,f'{label}: class identity does not match full candidate bytes')
 require(api.get('fields')==[],f'{label}: unexpected fields')
 methods=api.get('methods')
 require(isinstance(methods,list) and len(methods)==2,f'{label}: expected exactly two retained methods')
 rows=[]
 reports={}
 for i,item in enumerate(methods):
  mem=item.get('item',{}); name=mem.get('name',{}).get('escaped'); desc=mem.get('descriptor',{}).get('escaped')
  rows.append((name,desc)); require(mem.get('index')==i,f'{label}: method ordering/index mismatch')\n  require(item.get('markers')==[],f'{label}: unexpected method marker')\n  require(mem.get('access_flags')==(1 if i==0 else 8),f'{label}: constructor/static method flags changed')
  owner=mem.get('identity',{}).get('owner',{}).get('class_bytes',{})
  require(owner.get('digest')==digest and owner.get('length')==length,f'{label}: method identity owner mismatch')
  out=item.get('outcome',{}); rep=out.get('report',{})
  require(out.get('kind')=='recovered' and rep.get('outcome')=='produced',f'{label}: {name}{desc} not recovered')
  require(rep.get('quality')=='structured' and rep.get('representation')=='java' and rep.get('fallbacks')==[],f'{label}: {name}{desc} is not complete structured Java')
  reports[(name,desc)]=rep
 require(rows==[('<init>','()V'),('mixedExits','(I)I')],f'{label}: wrong members retained {rows!r}')
 return reports
def main():
 global CP_POOL
 ap=argparse.ArgumentParser()
 ap.add_argument('--execution',type=Path,required=True); ap.add_argument('--candidate-build',type=Path,required=True)
 ap.add_argument('--cli-sha256',required=True); ap.add_argument('--out',type=Path,required=True)
 a=ap.parse_args(); ep=a.execution.resolve(strict=True); out=ep.parent; op=a.out.resolve()
 require(not op.exists(),'verification output already exists')
 rec=json.loads(ep.read_text(encoding='utf-8'))
 require(rec.get('schema')=='candidate-continuation-full-class-replay-v1' and rec.get('original_source_available') is False,'execution schema/source boundary mismatch')
 require(rec.get('fixture',{}).get('path')==str(FIXTURE) and sha(FIXTURE)==FIXTURE_SHA and rec['fixture'].get('sha256')==FIXTURE_SHA,'candidate fixture SHA/path mismatch')
 require(rec.get('oracle_class',{}).get('path')==str(ORACLE) and sha(ORACLE)==ORACLE_SHA and rec['oracle_class'].get('sha256')==ORACLE_SHA,'full Join oracle class pin mismatch')
 require(rec.get('class_name')=='CandidateContinuation' and rec.get('oracle_class_name')=='Join','class names mismatch')
 require(sha(JOIN_RAW)==JOIN_RAW_SHA and pin_matches(rec['large_join_fallback_raw']),'complete Join fallback raw pin mismatch')
 
 join=json.loads(JOIN_RAW.read_text(encoding='utf-8')); jmethods=join.get('methods',[])
 require(join.get('class',{}).get('class_bytes')=={'digest':'f3f0c46b3b4933ff59edc09c38bc6af3c3721aade8a9f4e5a2eae6d191068718','length':726},'Join raw is not the complete frozen original class')
 require(join.get('declaration',{}).get('name')=='Join' and len(jmethods)==7,'large Join baseline must retain all seven members')
 outcomes={m.get('item',{}).get('name',{}).get('escaped'):m.get('outcome',{}).get('report',{}) for m in jmethods}
 for name in ('deepCatch','pick'):
  rep=outcomes.get(name,{})
  require(rep.get('quality')=='fallback' and rep.get('fallbacks'),'large Join expected fallback missing: '+name)
 require(sha(JAVAP)==JAVAP_SHA and pin_matches(rec['physical_javap']),'pinned physical javap rows mismatch')
 require(sha(GENERATION)==GENERATION_SHA and pin_matches(rec['generation']),'candidate generation record mismatch')
 gen=json.loads(GENERATION.read_text(encoding='utf-8'))
 require(gen.get('control_class',{}).get('sha256')==FIXTURE_SHA and gen['control_class'].get('bytes')==255 and gen.get('member_count')==2 and gen.get('code_bytes')==42 and gen.get('code_sha256')=='d8428ee78a6b587fcba9f412f119e33302f903019ac6bb8486269b5087363736' and gen.get('same_target_code') is True and gen.get('old_variant_sha256')==ORACLE_SHA and gen.get('old_goto_offset')==9 and gen.get('new_goto_offset')==6,'generation metadata does not prove the bounded complete-class control')
 physical=JAVAP.read_text(encoding='utf-8')
 require('public class CandidateContinuation' in physical and 'major version: 52' in physical,'javap owner/version is not the controlled Java 8 candidate')
 headers=[line.strip() for line in physical.splitlines() if line.strip() in ('public CandidateContinuation();','static int mixedExits(int);')]
 require(headers==['public CandidateContinuation();','static int mixedExits(int);'],'candidate javap member inventory is not the exact two-member class')
 CP_POOL=parse_cp(physical)
 block=re.search(r'static int mixedExits(int);([\s\S]*?)(?:\n  }|\Z)',physical)
 require(block is not None,'mixedExits javap block missing')
 bcis=[int(x) for x in re.findall(r'^\s*(\d+):',block.group(1),re.M)]
 require(len(bcis)==13 and len(set(bcis))==13 and set(bcis)==TARGET_BCIS,'mixedExits physical instruction BCI set mismatch')
 for needle in ('lookupswitch  { // 2','1: 28','2: 34','default: 37','goto          37','40: iload_1','41: ireturn'):
  require(needle in block.group(1),'physical target shape missing: '+needle)
 require(sha(BUILD_EVIDENCE)==BUILD_EVIDENCE_SHA,'build-evidence helper SHA mismatch')
 sys.path.insert(0,str(BUILD_EVIDENCE.parent)); from build_evidence import verify_build
 build_path=a.candidate_build.resolve(strict=True)
 require(rec.get('build_evidence',{}).get('path')==str(build_path) and rec['build_evidence'].get('sha256')==sha(build_path),'candidate build record path/SHA mismatch')
 build=verify_build(build_path,rec.get('cli'),a.cli_sha256)
 cli=Path(build['cli']['path']).resolve(strict=True)
 require(rec.get('candidate_build')==build and rec.get('cli',{}).get('sha256')==a.cli_sha256 and sha(cli)==a.cli_sha256,'actual v2 CLI/build pin mismatch')
 require(stat.S_IMODE(cli.stat().st_mode)==0o555,'candidate CLI mode differs from accepted build')
 build_helper=rec.get('build_evidence_helper',{})
 require(pin_matches(build_helper) and build_helper['sha256']==BUILD_EVIDENCE_SHA,'build helper pin mismatch')
 require(sha(GUARD)==GUARD_SHA and rec.get('guard',{}).get('path')==str(GUARD.resolve()) and rec['guard'].get('sha256')==GUARD_SHA,'resource guard pin mismatch')
 require(rec['guard'].get('poll_seconds')==1 and rec['guard'].get('minimum_free_bytes')==5*1024**3 and rec['guard'].get('maximum_target_bytes')==1024**3,'5GiB/1GiB/1sec guard values mismatch')
 require(sha(JDK_MANIFEST)==JDK_MANIFEST_SHA and pin_matches(rec['jdk_manifest']),'frozen JDK manifest mismatch')
 manifest=json.loads(JDK_MANIFEST.read_text()); tool_rows={x['leg']:x['jdk_tools'] for x in manifest['legs']}
 require(set(tool_rows)=={'javac8','javac23'},'expected both frozen JDK legs')
 tools={}
 for leg,rows in tool_rows.items():
  tools[leg]={}
  for kind,fact in rows.items():
   p=Path(fact['path']).resolve(strict=True); saved=rec['jdk_tools'][leg][kind]
   require(sha(p)==fact['sha256'] and pin_matches(saved) and Path(saved['path']).resolve()==p,'JDK pin mismatch '+leg+'/'+kind)
   tools[leg][kind]=p
 require(sha(JADX)==JADX_SHA and pin_matches(rec['jadx']),'JADX pin mismatch')
 commands=rec.get('commands')
 require(isinstance(commands,list) and len(commands)==19 and rec.get('status')=='all_19_commands_observed','not all 19 commands recorded')
 for i,row in enumerate(commands):
  require(row.get('label')==LABELS[i] and row.get('exit_code')==0 and row.get('guard_stop') is None,'command label/exit/guard mismatch '+str(i))
  require(Path(row.get('cwd','')).resolve()==ROOT and row.get('peak_target_bytes',0)<=1024**3 and row.get('free_bytes_after',0)>=5*1024**3,'command resource/cwd limits failed')
  raw_for(out,row,i)
 require(rec.get('full_class_methods_retained')==2,'collector did not retain exactly two candidate methods')
 cliargs=[str(cli),'class-source','--input',str(FIXTURE),'--class','CandidateContinuation','--policy','single-class','--release','8','--format','json']
 require(commands[0]['argv']==cliargs and commands[1]['argv']==cliargs+['--evidence','all'],'class-source command args differ')\n require(commands[2]['argv']==[str(JADX),'--no-res','--config','none','--threads-count','1','-d',str(out/'jadx-output'),str(out/'candidate.jar')],'JADX full-class invocation differs')
 api={}
 reports={}
 for profile,index in [('default',0),('all',1)]:
  raw=raw_for(out,commands[index],index)['stdout']; api[profile]=json.loads(raw.decode('utf-8'))
  saved=rec['apis'][profile]
  require(saved['methods']==2 and saved['outcome']=='performed','recorded full-class API lacks two methods')
  require(pin_matches(saved['api_json']) and json.loads(Path(saved['api_json']['path']).read_text())==api[profile],'saved API differs from raw')
  require(pin_matches(saved['source']) and Path(saved['source']['path']).read_text()==api[profile].get('text'),'saved source differs from raw API')
  digest=api[profile].get('class',{}).get('class_bytes',{}).get('digest'); length=api[profile].get('class',{}).get('class_bytes',{}).get('length')
  require(length==FIXTURE.stat().st_size,'API class length differs from fixture')
  reports[profile]=api_methods(api[profile],profile,digest,length)
  require('class CandidateContinuation' in api[profile].get('text','') and 'mixedExits' in api[profile]['text'],'full candidate source lacks expected members')
 require(api['default']['text']==api['all']['text'],'default/all source differs')
 for key in ('text','source_map','method','quality','representation','fallbacks','diagnostics'):
  require(reports['default'][('<init>','()V')].get(key)==reports['all'][('<init>','()V')].get(key) and reports['default'][('mixedExits','(I)I')].get(key)==reports['all'][('mixedExits','(I)I')].get(key),'default/all method '+key+' differs')
 mixed=reports['default'][('mixedExits','(I)I')]
 mapped=map_origins(mixed,api['default']['class']['class_bytes']['digest'],FIXTURE.stat().st_size,'mixedExits','(I)I')
 require(mapped==TARGET_BCIS,'source map does not close over all 13 unique target BCIs')
 constructor=reports['default'][('<init>','()V')]
 ctor_mapped=map_origins(constructor,api['default']['class']['class_bytes']['digest'],FIXTURE.stat().st_size,'<init>','()V')
 require(ctor_mapped=={0,1,4},'constructor source map does not cover its complete physical body')
 jar=out/'candidate.jar'; require(pin_matches(rec['jadx_input_jar']),'JADX input jar pin mismatch')
 import zipfile
 with zipfile.ZipFile(jar) as z: require(z.namelist()==['CandidateContinuation.class'] and z.read('CandidateContinuation.class')==FIXTURE.read_bytes(),'JADX input is not the complete pinned candidate')
 require(commands[2]['argv']==[str(JADX),'--no-res','--config','none','--threads-count','1','-d',str(out/'jadx-output'),str(jar)],'JADX full-class invocation differs')
 jsrc=out/'jadx-output/sources/defpackage/CandidateContinuation.java'
 require(jsrc.is_file() and pin_matches(rec['jadx_source']),'JADX whole-class source missing')
 expected=EXPECTED
 runtimes={}
 for i,(leg,profile) in enumerate(LEGS):
  row=rec['legs'][i]; work=out/leg/profile; classes=work/'classes'; empty=work/'empty'
  require((row.get('jdk'),row.get('profile'))==(leg,profile),'runtime leg order mismatch')
  runner=work/'Runner.java'; expected_class='Join' if profile=='original' else 'CandidateContinuation'
  expected_runner=('package defpackage;\n' if profile=='jadx' else '')+' '.join(
   [f'public class Runner {{ public static void main(String[] args) {{ {expected_class} instance = new {expected_class}();']+
   [f'System.out.println("{label}="+{expected_class}.mixedExits({val}));' for label,val in [('MIN','Integer.MIN_VALUE'),('-3','-3'),('-1','-1'),('0','0'),('1','1'),('2','2'),('3','3'),('4','4'),('MAX','Integer.MAX_VALUE')]]+['} }'])+'\n'
  require(runner.read_text()==expected_runner and pin_matches(row['runner']),'Runner source mismatch '+leg+'/'+profile)
  idx=3+i*2; c,r=commands[idx],commands[idx+1]
  require(row.get('compile_index')==idx and row.get('runtime_index')==idx+1 and row.get('runtime')==r,'runtime command indices mismatch')
  cp=classes if profile=='original' else empty
  flags=['-source','8','-target','8'] if leg=='javac8' else ['--release','8']
  src=work/'CandidateContinuation.java'
  sourceargs=[runner] if profile=='original' else [src,runner]
  expected_compile=list(map(str,[tools[leg]['javac'],'-J-Duser.language=en','-J-Duser.country=US','-encoding','UTF-8',*flags,'-g:none','-proc:none','-classpath',cp,'-sourcepath',empty,'-d',classes,*sourceargs]))
  require(c['argv']==expected_compile,'javac args mismatch '+leg+'/'+profile)
  main='defpackage.Runner' if profile=='jadx' else 'Runner'
  require(r['argv']==list(map(str,[tools[leg]['java'],'-Xverify:all','-cp',classes,main])),'runtime args mismatch '+leg+'/'+profile)
  if profile=='original':
   require((classes/'Join.class').read_bytes()==ORACLE.read_bytes(),'original profile did not use full Join.class')
   require(not src.exists(),'original oracle profile must not compile extracted method source')
   entries=sorted(p.relative_to(classes).as_posix() for p in classes.rglob('*.class'))
   require(entries==['Join.class','Runner.class'],'original profile class inventory changed')
  else:
   source_path=jsrc if profile=='jadx' else Path(rec['apis'][profile]['source']['path'])
   require(src.read_bytes()==source_path.read_bytes(),'candidate source copy differs from full-class source')
   expected_entries=['defpackage/CandidateContinuation.class','defpackage/Runner.class'] if profile=='jadx' else ['CandidateContinuation.class','Runner.class']
   require(sorted(p.relative_to(classes).as_posix() for p in classes.rglob('*.class'))==expected_entries,'candidate whole-class member inventory mismatch')
  raw=raw_for(out,r,i*2+4)['stdout']
  require(raw==expected and raw_for(out,r,i*2+4)['stderr']==b'','runtime mismatch '+leg+'/'+profile)
  runtimes[(leg,profile)]=(r['exit_code'],raw)
 for leg in ('javac8','javac23'):
  oracle=runtimes[(leg,'original')]
  require(all(runtimes[(leg,p)]==oracle for p in ('default','all','jadx')),leg+' CandidateContinuation differs from complete Join bytecode oracle')
 result={'schema':'candidate-continuation-full-class-replay-acceptance-v1','state':'accepted','execution':str(ep),'execution_sha256':sha(ep),'candidate_build':build,'candidate_fixture':rec['fixture'],'complete_join_oracle':rec['oracle_class'],'large_join_fallback_raw':rec['large_join_fallback_raw'],'physical_target_bcis':sorted(TARGET_BCIS),'constant_pool_utf8_closure':'all references resolve through Class/NameAndType to Utf8','candidate_methods':[{'name':'<init>','descriptor':'()V'},{'name':'mixedExits','descriptor':'(I)I'}],'command_count':19,'runtime_legs':8,'runtime_stdout':expected.decode(),'checks':['actual v2 CLI and build evidence','pinned full seven-member Join report retains deepCatch/pick fallback','candidate class contains exactly two methods','physical mixedExits target has 13 unique BCIs and closed UTF8 pool','source-map owner and BCI closure','full-class default/all/JADX compile and -Xverify:all behavior against complete Join class']}
 op.parent.mkdir(parents=True,exist_ok=True);op.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
 print(json.dumps({'state':'accepted','commands':19,'runtime_legs':8,'out':str(op)}));return 0
if __name__=='__main__': raise SystemExit(main())
