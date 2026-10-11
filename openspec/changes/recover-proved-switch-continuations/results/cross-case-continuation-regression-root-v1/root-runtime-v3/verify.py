#!/usr/bin/env python3
"""Independent acceptance for a complete two-member CandidateContinuation replay."""
from __future__ import annotations
import argparse, hashlib, json, re, stat, sys
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde').resolve()
BUILD_EVIDENCE=Path('/private/tmp/jarde-compound-final-replay-verifier-luna-v2/build_evidence.py')
BUILD_EVIDENCE_SHA='8b06e82de2fc696b3b82d32b444026b57a2d28f8c5cc51fef87d4ffcbe4ffec3'
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
MANIFEST=ROOT/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
MANIFEST_SHA='ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
JADX=Path('/opt/homebrew/bin/jadx').resolve()
JADX_SHA='64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
FIXTURE=ROOT/'openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/two-member-control/cross-case/CandidateContinuation.class'
FIXTURE_SHA='9aa523e64295cdf5738d1240ea2ba4b30b136cfec4a5b90cd362a61bfdb6c238'
CLASS_BLAKE3='06d3b07c87615ef02a521748823e792acd7df229773a2cccb90cb517e61f76e2'
JOIN_RAW=Path('/private/tmp/jarde-cross-case-continuation-diagnosis-root-v1/1.stdout.raw')
JOIN_RAW_SHA='c58ae7e7ab27ac53d2826e17c9e36a8c16f567ccb9c3790c25046cc831f1132a'
JAVAP=ROOT/'openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/two-member-control/1.stdout.raw'
JAVAP_SHA='37f039fe831d63981da1bf9dee369582a86d87e0653001eb5803a5046545407a'
GENERATION=ROOT/'openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/two-member-control/generation.json'
GENERATION_SHA='f956f5b2649201a1f61d1e49806c6afbf653868c3cbccb000316cfc1d9793a5f'
EXPECTED=b'MIN=30\n-3=30\n-1=30\n0=30\n1=30\n2=20\n3=30\n4=30\nMAX=30\n'
TARGET_BCIS={0,1,2,3,28,30,31,34,36,37,39,40,41}
LEGS=tuple((j,p) for j in ('javac8','javac23') for p in ('original','default','all','jadx'))
LABELS=['class-source-default','class-source-all','jadx-full-class']+[j+'-'+p+'-'+phase for j,p in LEGS for phase in ('compile','runtime')]
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
 require(row.get('index')==index,'command index drift '+str(index))
 got={}
 for key in ('stdout','stderr'):
  e=row.get('streams',{}).get(key)
  require(isinstance(e,dict) and pin_matches(e),'raw stream pin '+str(index)+'/'+key)
  require(Path(e['path']).resolve()==(out/'guard'/(str(index)+'.'+key+'.raw')).resolve(),'raw stream location drift')
  got[key]=Path(e['path']).read_bytes()
 return got
def map_origins(report,digest,length,name,descriptor):
 sm=report.get('source_map')
 require(isinstance(sm,dict) and isinstance(sm.get('segments'),list) and sm['segments'],'source map missing')
 bcis=set()
 text=report.get('text','')
 text_bytes=len(text.encode('utf-8'))
 for seg in sm['segments']:
  a,b=seg.get('start'),seg.get('end')
  require(isinstance(a,int) and isinstance(b,int) and 0<=a<=b<=text_bytes,'source-map byte span outside UTF-8 method text')
  require(bool(text.encode('utf-8')[a:b].decode('utf-8')),'source-map range is empty or splits UTF-8')
  origin=seg.get('origin',{})
  origins=[]
  if isinstance(origin.get('primary'),dict): origins.append(origin['primary'])
  origins.extend(x for x in origin.get('derived',[]) if isinstance(x,dict))
  require(bool(origins),'source-map segment has no physical owner')
  for o in origins:
   m=o.get('method',{})
   owner=m.get('owner',{}).get('class_bytes',{})
   require(owner.get('digest')==digest and owner.get('length')==length,'source-map owner differs from candidate class')
   require(byte_text(m.get('name',[]))==name and byte_text(m.get('descriptor',[]))==descriptor,'source-map method owner mismatch')
   bci=o.get('bci')
   require(isinstance(bci,int),'source-map BCI missing')
   bcis.add(bci)
 return bcis
def api_methods(api,label):
 require(api.get('outcome')=='performed',label+': whole-class API not performed')
 cls=api.get('class',{}).get('class_bytes',{})
 require(cls.get('digest')==CLASS_BLAKE3 and cls.get('length')==255,label+': API class identity differs from frozen class BLAKE3/length')
 require(api.get('fields')==[],label+': unexpected fields')
 methods=api.get('methods')
 require(isinstance(methods,list) and len(methods)==2,label+': expected exactly two retained methods')
 rows=[]
 reports={}
 for i,item in enumerate(methods):
  mem=item.get('item',{})
  name=mem.get('name',{}).get('escaped')
  desc=mem.get('descriptor',{}).get('escaped')
  rows.append((name,desc))
  require(mem.get('index')==i,label+': member ordering/index mismatch')
  require(item.get('markers')==[],label+': unexpected method marker')
  require(mem.get('access_flags')==(1 if i==0 else 8),label+': member access flags changed')
  owner=mem.get('identity',{}).get('owner',{}).get('class_bytes',{})
  require(owner.get('digest')==CLASS_BLAKE3 and owner.get('length')==255,label+': method owner identity mismatch')
  outcome=item.get('outcome',{})
  rep=outcome.get('report',{})
  require(outcome.get('kind')=='recovered' and rep.get('outcome')=='produced',label+': method not recovered')
  require(rep.get('quality')=='structured' and rep.get('representation')=='java' and rep.get('fallbacks')==[],label+': method output is incomplete/fallback')
  reports[(name,desc)]=rep
 require(rows==[('<init>','()V'),('mixedExits','(I)I')],label+': wrong retained members '+repr(rows))
 return reports
def main():
 ap=argparse.ArgumentParser()
 ap.add_argument('--execution',type=Path,required=True)
 ap.add_argument('--candidate-build',type=Path,required=True)
 ap.add_argument('--cli-sha256',required=True)
 ap.add_argument('--out',type=Path,required=True)
 a=ap.parse_args()
 ep=a.execution.resolve(strict=True); out=ep.parent; op=a.out.resolve()
 require(not op.exists(),'verification output already exists')
 rec=json.loads(ep.read_text(encoding='utf-8'))
 require(rec.get('schema')=='candidate-continuation-full-class-replay-v2','execution schema mismatch')
 require(rec.get('fixture',{}).get('path')==str(FIXTURE) and sha(FIXTURE)==FIXTURE_SHA and rec['fixture'].get('sha256')==FIXTURE_SHA,'candidate fixture SHA/path mismatch')
 require(rec.get('canonical_original',{}).get('path')==str(FIXTURE) and pin_matches(rec['canonical_original']),'canonical original must be the complete CandidateContinuation class')
 require(rec.get('fixture_blake3')==CLASS_BLAKE3,'frozen independent class BLAKE3 mismatch')
 require(pin_matches(rec.get('large_join_fallback_raw',{})) and sha(JOIN_RAW)==JOIN_RAW_SHA,'read-only large Join raw pin mismatch')
 join=json.loads(JOIN_RAW.read_text(encoding='utf-8'))
 require(join.get('declaration',{}).get('name')=='Join' and len(join.get('methods',[]))==7,'large Join record must retain its seven-member scope')
 joined={m.get('item',{}).get('name',{}).get('escaped'):m.get('outcome',{}).get('report',{}) for m in join['methods']}
 for name in ('deepCatch','pick'):
  require(joined.get(name,{}).get('quality')=='fallback' and joined[name].get('fallbacks'),'large Join fallback boundary record missing '+name)
 require(sha(JAVAP)==JAVAP_SHA and pin_matches(rec.get('physical_javap',{})),'physical javap evidence pin mismatch')
 require(sha(GENERATION)==GENERATION_SHA and pin_matches(rec.get('generation',{})),'generation evidence pin mismatch')
 gen=json.loads(GENERATION.read_text(encoding='utf-8'))
 control=gen.get('control_class',{})
 require(control.get('sha256')==FIXTURE_SHA and control.get('bytes')==255 and gen.get('member_count')==2 and gen.get('code_bytes')==42 and gen.get('code_sha256')=='d8428ee78a6b587fcba9f412f119e33302f903019ac6bb8486269b5087363736' and gen.get('same_target_code') is True and gen.get('old_goto_offset')==9 and gen.get('new_goto_offset')==6,'generation record does not bind the whole-class control')
 physical=JAVAP.read_text(encoding='utf-8')
 require('public class CandidateContinuation' in physical and 'major version: 52' in physical,'physical class/version mismatch')
 require('public CandidateContinuation();' in physical and 'static int mixedExits(int);' in physical,'physical two-method inventory missing')
 block=re.search(r'static int mixedExits\(int\);([\s\S]*?)(?:\n  }|\Z)',physical)
 require(block is not None,'mixedExits physical block missing')
 bcis=[int(x) for x in re.findall(r'^[ \t]*(\d+):[ \t]+(?:iconst(?:_\d+)?|istore(?:_\d+)?|iload(?:_\d+)?|lookupswitch|bipush|goto|ireturn)\b',block.group(1),re.M)]
 require(len(bcis)==13 and len(set(bcis))==13 and set(bcis)==TARGET_BCIS,'physical BCI inventory mismatch')
 for needle in ('lookupswitch  { // 2','1: 28','2: 34','default: 37','goto          37','40: iload_1','41: ireturn'):
  require(needle in block.group(1),'physical target shape missing '+needle)
 require(sha(BUILD_EVIDENCE)==BUILD_EVIDENCE_SHA,'build-evidence helper SHA mismatch')
 sys.path.insert(0,str(BUILD_EVIDENCE.parent))
 from build_evidence import verify_build
 build_path=a.candidate_build.resolve(strict=True)
 require(rec.get('build_evidence',{}).get('path')==str(build_path) and rec['build_evidence'].get('sha256')==sha(build_path),'build evidence path/SHA mismatch')
 build=verify_build(build_path,rec.get('cli'),a.cli_sha256)
 cli=Path(build['cli']['path']).resolve(strict=True)
 require(rec.get('candidate_build')==build and rec.get('cli',{}).get('sha256')==a.cli_sha256 and sha(cli)==a.cli_sha256,'actual CLI/build pin mismatch')
 require(stat.S_IMODE(cli.stat().st_mode)==0o555,'candidate CLI mode mismatch')
 require(pin_matches(rec.get('build_evidence_helper',{})) and rec['build_evidence_helper']['sha256']==BUILD_EVIDENCE_SHA,'build helper pin mismatch')
 require(sha(GUARD)==GUARD_SHA and rec.get('guard',{}).get('path')==str(GUARD.resolve()) and rec['guard'].get('sha256')==GUARD_SHA,'guard source pin mismatch')
 require(rec['guard'].get('poll_seconds')==1 and rec['guard'].get('minimum_free_bytes')==5*1024**3 and rec['guard'].get('maximum_target_bytes')==1024**3,'guard limit mismatch')
 require(sha(MANIFEST)==MANIFEST_SHA and pin_matches(rec.get('jdk_manifest',{})),'JDK manifest pin mismatch')
 manifest=json.loads(MANIFEST.read_text(encoding='utf-8'))
 tool_rows={x['leg']:x['jdk_tools'] for x in manifest['legs']}
 require(set(tool_rows)=={'javac8','javac23'},'expected both frozen JDK legs')
 for leg,rows in tool_rows.items():
  for kind,fact in rows.items():
   saved=rec['jdk_tools'][leg][kind]; p=Path(fact['path']).resolve(strict=True)
   require(sha(p)==fact['sha256'] and pin_matches(saved) and Path(saved['path']).resolve()==p,'JDK pin mismatch '+leg+'/'+kind)
 require(sha(JADX)==JADX_SHA and pin_matches(rec.get('jadx',{})),'JADX pin mismatch')
 require(len(rec.get('commands',[]))==19,'expected exact 19-command replay')
 require([x.get('label') for x in rec['commands']]==LABELS,'command order/argv labels mismatch')
 cli_path=rec['cli']['path']
 out_path=str(out)
 api0=[cli_path,'class-source','--input',str(FIXTURE),'--class','CandidateContinuation','--policy','single-class','--release','8','--format','json']
 api1=api0+['--evidence','all']
 jar_path=str(out/'candidate.jar')
 jadx_argv=[str(JADX),'--no-res','--config','none','--threads-count','1','-d',str(out/'jadx-output'),jar_path]
 expected_argv=[api0,api1,jadx_argv]
 for jdk,profile in LEGS:
  item=next(x for x in rec['legs'] if x.get('jdk')==jdk and x.get('profile')==profile)
  work=Path(item['runner']['path']).parent; classes=Path(item['classes_dir']); empty=Path(item['empty_dir']); runpath=Path(item['runner']['path'])
  tool=rec['jdk_tools'][jdk]
  flags=['-source','8','-target','8'] if jdk=='javac8' else ['--release','8']
  src=[str(runpath)] if profile=='original' else [str(work/'CandidateContinuation.java'),str(runpath)]
  cp=classes if profile=='original' else empty
  compile_argv=[tool['javac']['path'],'-J-Duser.language=en','-J-Duser.country=US','-encoding','UTF-8',*flags,'-g:none','-proc:none','-classpath',str(cp),'-sourcepath',str(empty),'-d',str(classes),*src]
  main='defpackage.Runner' if profile=='jadx' else 'Runner'
  runtime_argv=[tool['java']['path'],'-Xverify:all','-cp',str(classes),main]
  expected_argv.extend([compile_argv,runtime_argv])
 require([x.get('argv') for x in rec['commands']]==expected_argv,'full command argv differs from the frozen 19-command matrix')
 for i,row in enumerate(rec['commands']):
  require(row.get('guard_stop') is None and row.get('exit_code')==0,'guard/command failure at '+str(i))
  raw=raw_for(out,row,i)
  require(raw['stderr']==b'','stderr was not empty at '+str(i))
 for p,label in [('default','default'),('all','all')]:
  meta=rec.get('apis',{}).get(p,{})
  api=json.loads(Path(meta['api_json']['path']).read_text(encoding='utf-8'))
  raw_api=json.loads(raw_for(out,rec['commands'][meta['command_index']],meta['command_index'])['stdout'])
  require(api==raw_api,'saved API differs from actual CLI raw stdout')
  require(Path(meta['source']['path']).read_text(encoding='utf-8')==api['text'],'saved full-class source differs from actual API')
  require(pin_matches(meta['api_json']) and pin_matches(meta['source']),'saved API/source pin mismatch')
  reports=api_methods(api,label)
  require(meta.get('methods')==2,'collector method count mismatch')
  ctor=map_origins(reports[('<init>','()V')],CLASS_BLAKE3,255,'<init>','()V')
  require(ctor=={0,1,4},label+': constructor map BCI closure mismatch')
  mixed=map_origins(reports[('mixedExits','(I)I')],CLASS_BLAKE3,255,'mixedExits','(I)I')
  require(mixed==TARGET_BCIS,label+': mixedExits source map does not close over all 13 physical BCIs')
  if p=='default': default_api=api; default_source=Path(meta['source']['path']).read_bytes()
  else:
   require(Path(meta['source']['path']).read_bytes()==default_source,'default/all complete source differs')
   for left,right in zip(default_api['methods'],api['methods']):
    require(left['item']==right['item'],'default/all complete member identity differs')
    for key in ('text','source_map','method','quality','representation','fallbacks','diagnostics'):
     require(left['outcome']['report'][key]==right['outcome']['report'][key],'default/all '+key+' differs')
 require(rec.get('full_class_methods_retained')==2,'whole class did not retain both members')
 source_records={x['profile']:x for x in rec['legs'] if x['jdk']=='javac8'}
 for jdk,profile in LEGS:
  matches=[x for x in rec['legs'] if x.get('jdk')==jdk and x.get('profile')==profile]
  require(len(matches)==1,'runtime leg missing/duplicated '+jdk+'/'+profile)
  item=matches[0]; ri=item['runtime_index']; ci=item['compile_index']
  compile_row=rec['commands'][ci]; runtime_row=rec['commands'][ri]
  argv=compile_row['argv']; rargv=runtime_row['argv']
  require(argv[0]==rec['jdk_tools'][jdk]['javac']['path'],'compile did not use pinned javac')
  require(rargv[0]==rec['jdk_tools'][jdk]['java']['path'] and '-Xverify:all' in rargv,'runtime did not use pinned java with full verification')
  require(raw_for(out,runtime_row,ri)['stdout']==EXPECTED,'runtime output mismatch '+jdk+'/'+profile)
  require(Path(item['runner']['path']).is_file() and pin_matches(item['runner']),'Runner source pin mismatch')
  runner=Path(item['runner']['path']).read_text(encoding='utf-8')
  require(runner.count('new CandidateContinuation()')==1 and runner.count('mixedExits(')==9,'Runner must construct once and execute nine inputs')
  entries=sorted(p.relative_to(Path(item['classes_dir'])).as_posix() for p in Path(item['classes_dir']).rglob('*.class'))
  wanted=['defpackage/CandidateContinuation.class','defpackage/Runner.class'] if profile=='jadx' else ['CandidateContinuation.class','Runner.class']
  require(entries==wanted,'full generated class inventory differs '+jdk+'/'+profile)
  if profile=='original':
   classfile=Path(item['classes_dir'])/'CandidateContinuation.class'
   require(classfile.is_file() and sha(classfile)==FIXTURE_SHA,'original runtime class is not the complete pinned control')
  else:
   source=Path(item['source']['path'])
   require(pin_matches(item['source']),'compiled source pin mismatch')
   if profile in ('default','all'):
    require(source.read_bytes()==default_source,'whole-class compiler source differs from accepted API output')
   else:
    require(source.is_file() and 'class CandidateContinuation' in source.read_text(encoding='utf-8'),'JADX whole-class source missing candidate class')
 require(len(rec['legs'])==8,'expected eight full runtime legs')
 result={'status':'accepted','commands':19,'runtime_legs':8,'inputs_per_leg':9,'members':2,'mixedExits_bcis':sorted(TARGET_BCIS),'candidate_sha256':FIXTURE_SHA,'candidate_blake3':CLASS_BLAKE3,'original_runtime':'complete CandidateContinuation.class','large_Join':'read-only seven-member fallback-boundary evidence'}
 op.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
 print(json.dumps(result,sort_keys=True))
if __name__=='__main__': main()
