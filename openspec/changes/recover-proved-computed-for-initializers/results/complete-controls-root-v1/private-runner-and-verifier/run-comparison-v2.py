#!/usr/bin/env python3
"""Fresh full-class JADX and Jarde source/runtime comparison; deliberately not executed by preparer."""
from __future__ import annotations
import argparse, hashlib, json, os, shutil, stat, subprocess, sys, time, zipfile
from pathlib import Path

OUT_DEFAULT = Path('/private/tmp/jarde-computed-for-controls-jadx-root-v2')
REPO = Path('/Users/lordcasser/workspace/projects/jarde')
JADX_MANIFEST = Path('/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-10/cf07-loop-latch-baseline/baseline-root-v2/manifest.json')
PLAN = Path('/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-switch-continuations/results/candidate-full-class-root-v1/jarde-cf12-ten-fixture-replay-plan-luna-v1.json')
CONTROLS_EXEC = Path('/private/tmp/jarde-computed-for-controls-root-v1/execution.json')
DIRECT_EXEC = Path('/private/tmp/jarde-computed-for-direct-return-root-v1/execution.json')
CONTROLS_SRC = Path('/private/tmp/jarde-computed-for-controls-luna-v1/ComputedForControls.java')
CONTROLS_RUNNER = Path('/private/tmp/jarde-computed-for-controls-luna-v1/ComputedForControlsRunner.java')
DIRECT_SRC = Path('/private/tmp/jarde-computed-for-controls-independent-luna-v1/v2/ComputedForDirectReturn.java')
DIRECT_RUNNER = Path('/private/tmp/jarde-computed-for-controls-independent-luna-v1/v2/ComputedForDirectReturnRunner.java')
CLI = Path('/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2')
CLI_SHA = 'b081ca53080d0a41f00bf8f5b924499a2c781f413746fc9facb7862ef8b9cfe0'
JDKS = {
 'javac8': {'home': '/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home', 'javac_sha':'598601455e9f81ef05225c014743aaafa993d572afc7f83f5e7934ff10f9dddd', 'java_sha':'3167d94b1b56e1330e25e48a92bba9cba423ca578525c4ae66e8b5a06fb5dbb5'},
 'javac23': {'home': '/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home', 'javac_sha':'a3e79462d70cb70c34b85ab94ae615328d1cbe7ac6a2c69635288ae6cbb902a8', 'java_sha':'b79b8bac2b2a2c2b4d0dcb9c3981d477bd58bc95c5c4404dfb40a37e581497a1'},
}
EXPECTED_INPUTS = {
 ('controls','javac8','original'): '4c253eb463a8e5bf105c2814e080cf31eb31235dca1a8b98ed7c8d2f3e471dff',
 ('controls','javac8','pending'): 'f28859e6823aa643cedeb7ead216a7ee5debcd9ba35d241439c3b60d7435ee81',
 ('controls','javac23','original'): '1baf82bd5b2e9c2e79777e6393e49c3dca433344c223ad30b94192a28295ccad',
 ('controls','javac23','pending'): '325dee2da3e623b3bac2bef1219b8dfca95a166d9f58315f65003859d277f14e',
 ('direct-return','javac8','original'): '8f68e9bf708ef04bbe726646bfd227fcbe51e80fdda4bd93da04f6d53d644735',
 ('direct-return','javac23','original'): '8badd52db458312f3dfc4a60f591b85eeef502a5d5afaba5ef0a21d8505f4efe',
}
ENV_CLEAN = ['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH']


def sha(b: bytes) -> str: return hashlib.sha256(b).hexdigest()
def file_pin(p: Path) -> dict:
    b=p.read_bytes(); return {'path':str(p),'bytes':len(b),'sha256':sha(b)}
def check_pin(p: Path, expected: str, label: str) -> dict:
    fact=file_pin(p)
    if fact['sha256'] != expected: raise ValueError(f'{label} SHA-256 mismatch: {fact}')
    return fact
def load_json(p: Path): return json.loads(p.read_text(encoding='utf-8'))
def atomic_json(p: Path, obj):
    tmp=p.with_suffix(p.suffix+'.tmp'); tmp.write_text(json.dumps(obj,indent=2,sort_keys=True)+'\n',encoding='utf-8'); tmp.replace(p)

class Run:
    def __init__(self, out: Path):
        if out.exists(): raise ValueError(f'refusing existing output directory: {out}')
        out.mkdir(parents=True); self.out=out; self.records=[]
    def command(self,label,argv,*,cwd,java_home,allow_nonzero=False):
        env=os.environ.copy()
        for k in ENV_CLEAN: env.pop(k,None)
        env['JAVA_HOME']=str(java_home)
        env['PATH']=str(Path(java_home)/'bin')+os.pathsep+env.get('PATH','')
        started=time.time(); t0=time.monotonic()
        try:
            proc=subprocess.run([str(x) for x in argv],cwd=str(cwd),env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=90,check=False)
            out,err,code=proc.stdout,proc.stderr,proc.returncode
        except subprocess.TimeoutExpired as e:
            out=e.stdout or b''; err=e.stderr or b''; code=None
        (self.out/f'{label}.stdout.raw').write_bytes(out)
        (self.out/f'{label}.stderr.raw').write_bytes(err)
        row={'label':label,'argv':[str(x) for x in argv],'cwd':str(cwd),'java_home':str(java_home),'removed_env_keys':ENV_CLEAN,
             'started_unix':started,'duration_seconds':time.monotonic()-t0,'exit':code,'timed_out':code is None,
             'stdout':{'path':f'{label}.stdout.raw','bytes':len(out),'sha256':sha(out)},
             'stderr':{'path':f'{label}.stderr.raw','bytes':len(err),'sha256':sha(err)}}
        self.records.append(row); atomic_json(self.out/'execution.json',{'status':'running','commands':self.records})
        if code is None: raise TimeoutError(f'{label} timed out; raw streams retained')
        if code != 0 and not allow_nonzero: row['unexpected_nonzero']=True
        return row,out,err

def source_text(path: Path) -> str:
    return path.read_text(encoding='utf-8')

PACKAGE_RE = __import__('re').compile(r'(?m)^[ \t]*package\s+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)\s*;[ \t]*$')

def package_decl(source: bytes, label: str) -> str:
    text = source.decode('utf-8')
    matches = PACKAGE_RE.findall(text)
    if len(matches) != 1:
        raise ValueError(f'{label}: expected exactly one package declaration, found {matches!r}')
    return matches[0]

def javac_argv(leg, source, runner, classes):
    tool=str(Path(JDKS[leg]['home'])/'bin'/'javac')
    prefix=[tool,'-source','8','-target','8'] if leg=='javac8' else [tool,'--release','8']
    return prefix+['-g:none','-proc:none','-d',str(classes),str(source),str(runner)]
def java_argv(leg,classes,main): return [str(Path(JDKS[leg]['home'])/'bin'/'java'),'-Xverify:all','-cp',str(classes),main]
def case_path(out, name):
    p=out/name; p.mkdir(parents=True,exist_ok=True); return p

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--output',type=Path,default=OUT_DEFAULT)
    args=ap.parse_args(); out=args.output.resolve(); run=Run(out)
    execution={'status':'preflight','runner':file_pin(Path(__file__).resolve()),'inputs':{},'tool_pins':{},'dependency_manifest':{},'cases':[],'commands':run.records,
      'scope':{'jadx':'fresh full class decompile; source compiled with its matching runner on both JDKs; runtime uses -Xverify:all',
               'jadx_dependency_pins':'the 63 manifest jars are independently pin-checked; JADX execution uses the separately SHA-pinned Homebrew 1.5.6 launcher',
               'jarde':'fresh single-class default/all full-class source; compile failures are recorded whole-class and are never treated as runtime',
               'controls':'computed-for current source compile only; no runtime claim',
               'direct-return':'computed-for-direct-return full source compile and runtime when compile succeeds',
               'jadx_runner_adapter':'prepend the unique package declaration from the unchanged complete JADX source to the unchanged original Runner source; record before/prefix/after hashes; execute the qualified main'}}
    atomic_json(out/'execution.json',execution)
    # Freeze the provenance manifests and all 63 dependency jars before invoking tools.
    for p in [JADX_MANIFEST,PLAN,CONTROLS_EXEC,DIRECT_EXEC,CONTROLS_SRC,CONTROLS_RUNNER,DIRECT_SRC,DIRECT_RUNNER,CLI]:
        execution['inputs'][str(p)]=file_pin(p)
    if execution['inputs'][str(CLI)]['sha256'] != CLI_SHA: raise ValueError('frozen Jarde CLI SHA mismatch')
    jm=load_json(JADX_MANIFEST); jpin=jm['jadx']; launcher=Path(jpin['resolved_launcher'])
    execution['tool_pins']['jadx_launcher']=check_pin(launcher,jpin['sha256'],'JADX launcher')
    execution['tool_pins']['jadx_symlink']= {'path':jpin['launcher'],'resolved':str(launcher)}
    if jpin['expected_version']!='1.5.6': raise ValueError('JADX version pin is not 1.5.6')
    plan=load_json(PLAN); deps=[]
    for group in ('test_sdk_jars','jadx_product_jars'):
        for ref in plan['shared_pins'][group]:
            p=Path(ref['path']); fact=file_pin(p)
            if fact['bytes']!=ref['bytes'] or fact['sha256']!=ref['sha256']: raise ValueError(f'63-jar dependency pin mismatch: {p}')
            deps.append(fact)
    if len(deps)!=63: raise ValueError(f'expected 63 pinned jars, got {len(deps)}')
    execution['dependency_manifest']={'path':str(PLAN),'sha256':sha(PLAN.read_bytes()),'jar_count':len(deps),'jars':deps}
    ce=load_json(CONTROLS_EXEC); de=load_json(DIRECT_EXEC)
    controls_runner_sha='ff5276bd232a0e3861d18e52b90e51a45738947e8e80ba39b42eb4120cf3e534'
    direct_runner_sha='a1c5205a662d19d34e1ab4f329753f59dcd366f2ebac221c52d9c2089add89dc'
    check_pin(CONTROLS_RUNNER,controls_runner_sha,'controls runner source'); check_pin(DIRECT_RUNNER,direct_runner_sha,'direct runner source')
    for leg,info in JDKS.items():
        for tool,expected in [('javac',info['javac_sha']),('java',info['java_sha'])]:
            execution['tool_pins'][leg+'.'+tool]=check_pin(Path(info['home'])/'bin'/tool,expected,leg+'.'+tool)
    execution['tool_pins']['jarde_cli']=check_pin(CLI,CLI_SHA,'frozen Jarde CLI')
    cases=[]
    # Four root-v1 controls and both original direct-return classes.
    for fixture, leg, side, class_path, class_name, src, runner_src, main_class, expected_sha in [
      ('controls','javac8','original','/private/tmp/jarde-computed-for-controls-root-v1/javac8/original/ComputedForControls.class','ComputedForControls',CONTROLS_SRC,CONTROLS_RUNNER,'ComputedForControlsRunner',EXPECTED_INPUTS[('controls','javac8','original')]),
      ('controls','javac8','pending','/private/tmp/jarde-computed-for-controls-root-v1/javac8/pending/ComputedForControls.class','ComputedForControls',CONTROLS_SRC,CONTROLS_RUNNER,'ComputedForControlsRunner',EXPECTED_INPUTS[('controls','javac8','pending')]),
      ('controls','javac23','original','/private/tmp/jarde-computed-for-controls-root-v1/javac23/original/ComputedForControls.class','ComputedForControls',CONTROLS_SRC,CONTROLS_RUNNER,'ComputedForControlsRunner',EXPECTED_INPUTS[('controls','javac23','original')]),
      ('controls','javac23','pending','/private/tmp/jarde-computed-for-controls-root-v1/javac23/pending/ComputedForControls.class','ComputedForControls',CONTROLS_SRC,CONTROLS_RUNNER,'ComputedForControlsRunner',EXPECTED_INPUTS[('controls','javac23','pending')]),
      ('direct-return','javac8','original','/private/tmp/jarde-computed-for-direct-return-root-v1/javac8/classes/ComputedForDirectReturn.class','ComputedForDirectReturn',DIRECT_SRC,DIRECT_RUNNER,'ComputedForDirectReturnRunner',EXPECTED_INPUTS[('direct-return','javac8','original')]),
      ('direct-return','javac23','original','/private/tmp/jarde-computed-for-direct-return-root-v1/javac23/classes/ComputedForDirectReturn.class','ComputedForDirectReturn',DIRECT_SRC,DIRECT_RUNNER,'ComputedForDirectReturnRunner',EXPECTED_INPUTS[('direct-return','javac23','original')]),
    ]:
        srcpath=Path(class_path); pin=check_pin(srcpath,expected_sha,f'{fixture}/{leg}/{side} class')
        case={'id':f'{fixture}-{leg}-{side}','fixture':fixture,'source_jdk':leg,'side':side,'class_name':class_name,'main_class':main_class,
              'class_input':pin,'runner_source':file_pin(runner_src),'runner_source_path':str(runner_src),'source_input':file_pin(src),'source_input_path':str(src)}
        # Pin original raw oracle generated by the already verified runtime. Both controls legs share exact nine rows.
        if fixture=='controls':
            oracle=ce['runtime_pairs'][0]['stdout'].encode('utf-8')
            expected_oracle_path='/private/tmp/jarde-computed-for-controls-root-v1/javac8-original-runtime.stdout.raw'
        else:
            expected_oracle_path=f'/private/tmp/jarde-computed-for-direct-return-root-v1/{leg}-runtime.stdout.raw'
            oracle=Path(expected_oracle_path).read_bytes()
        oracle_path=Path(expected_oracle_path)
        if oracle_path.exists():
            recorded=oracle_path.read_bytes()
            if recorded!=oracle: raise ValueError(f'{fixture} oracle raw file differs from verified manifest output: {oracle_path}')
        else: recorded=oracle
        oracle_file=case_path(out,'oracles')/(case['id']+'.stdout.raw'); oracle_file.write_bytes(recorded)
        case['oracle']={'source':expected_oracle_path,'source_sha256':sha(recorded),'copied_path':str(oracle_file),'bytes':len(recorded)}
        if fixture=='controls' and oracle.decode('utf-8').count('\n')!=9: raise ValueError('controls oracle is not nine rows')
        cases.append((case,src,runner_src,srcpath))
    # Verify every previously captured controls runtime stream, not just the chosen canonical copy.
    control_oracle_paths=[
      Path('/private/tmp/jarde-computed-for-controls-root-v1/javac8-original-runtime.stdout.raw'),
      Path('/private/tmp/jarde-computed-for-controls-root-v1/javac8-pending-runtime.stdout.raw'),
      Path('/private/tmp/jarde-computed-for-controls-root-v1/javac23-original-runtime.stdout.raw'),
      Path('/private/tmp/jarde-computed-for-controls-root-v1/javac23-pending-runtime.stdout.raw'),
    ]
    control_oracles=[{'path':str(p),**file_pin(p)} for p in control_oracle_paths]
    if len({Path(x['path']).read_bytes() for x in control_oracles})!=1:
        raise ValueError('the four pre-existing controls runtime outputs are not byte-identical')
    if Path(control_oracle_paths[0]).read_bytes()!=ce['runtime_pairs'][0]['stdout'].encode('utf-8'):
        raise ValueError('controls raw runtime oracle differs from the verified execution manifest')
    execution['oracle_crosschecks']={'controls_existing_runtime_raws':control_oracles,
      'controls_expected_rows':9,'direct_return_existing_runtime_raws':[{'path':f'/private/tmp/jarde-computed-for-direct-return-root-v1/{leg}-runtime.stdout.raw',**file_pin(Path(f'/private/tmp/jarde-computed-for-direct-return-root-v1/{leg}-runtime.stdout.raw'))} for leg in ('javac8','javac23')]}
    if Path('/private/tmp/jarde-computed-for-direct-return-root-v1/javac8-runtime.stdout.raw').read_bytes()!=Path('/private/tmp/jarde-computed-for-direct-return-root-v1/javac23-runtime.stdout.raw').read_bytes():
        raise ValueError('direct-return original runtime outputs are not byte-identical across JDKs')
    execution['cases']=[x[0] for x in cases]; atomic_json(out/'execution.json',execution)

    # Version probe uses the real, hash-pinned 1.5.6 launcher with JDK23 and a clean Java option environment.
    ver,verout,_=run.command('jadx-version',[jpin['resolved_launcher'],'--version'],cwd=REPO,java_home=JDKS['javac23']['home'])
    if ver['exit']!=0 or verout.decode('utf-8').strip()!='1.5.6': raise ValueError('fresh JADX version is not 1.5.6')

    # Build one deterministic one-member jar for each frozen class; decompile all six complete class inputs.
    jadx_sources={}
    for case,src,runner_src,classfile in cases:
        key=case['id']; d=case_path(out,'jadx/'+key); jar=d/'input.jar'
        info=zipfile.ZipInfo(case['class_name']+'.class',(1980,1,1,0,0,0)); info.compress_type=zipfile.ZIP_DEFLATED; info.create_system=3; info.external_attr=(stat.S_IFREG|0o644)<<16
        with zipfile.ZipFile(jar,'w') as z: z.writestr(info,classfile.read_bytes(),compress_type=zipfile.ZIP_DEFLATED,compresslevel=9)
        jarref=file_pin(jar); jarref['only_member']=case['class_name']+'.class'; case['jadx_jar']=jarref
        od=d/'out'; argv=[jpin['resolved_launcher'],'--no-res','--config','none','--threads-count','1','-d',od,jar]
        row,_,_=run.command('jadx-'+key,argv,cwd=REPO,java_home=JDKS['javac23']['home'])
        srcs=sorted(od.rglob(case['class_name']+'.java')) if od.exists() else []
        case['jadx_decompile_exit']=row['exit']; case['jadx_generated_sources']=[file_pin(p) for p in srcs]
        if row['exit']==0 and len(srcs)!=1: case['jadx_source_discovery_error']=f'expected one complete {case["class_name"]}.java; found {len(srcs)}'
        if len(srcs)==1:
            jadx_sources[key]=srcs[0]
            # The six v1 source/package bindings were independently inspected and all declare defpackage.
            package=package_decl(srcs[0].read_bytes(),key)
            if package!='defpackage': raise ValueError(f'{key}: fresh JADX package differs from audited defpackage binding: {package}')
            case['jadx_source_package']=package
            case['jadx_source_path']=str(srcs[0]);
    atomic_json(out/'execution.json',execution)

    # Full source plus matching runner, each decompiled class cross-compiled and verified on both JDKs.
    for case,src,runner_src,classfile in cases:
        key=case['id']; decompiled=jadx_sources.get(key)
        if decompiled is None: continue
        for leg in ('javac8','javac23'):
            dest=case_path(out,f'jadx-build/{key}/{leg}'); classes=dest/'classes'; classes.mkdir()
            copied_runner=dest/runner_src.name
            runner_before=runner_src.read_bytes()
            runner_pkgs=PACKAGE_RE.findall(runner_before.decode('utf-8'))
            if runner_pkgs: raise ValueError(f'{key}: original Runner must have no package declaration before adaptation: {runner_pkgs}')
            source_bytes=decompiled.read_bytes(); package=package_decl(source_bytes,key)
            prefix=f'package {package};\n'.encode('utf-8')
            adapted_runner=prefix+runner_before
            copied_runner.write_bytes(adapted_runner)
            copied_source=dest/decompiled.name; copied_source.write_bytes(source_bytes)
            adapter={'source_package':package,'main_class_original':case['main_class'],'main_class_qualified':package+'.'+case['main_class'],
                     'runner_before':{'bytes':len(runner_before),'sha256':sha(runner_before)},'exact_prefix':prefix.decode('utf-8'),
                     'prefix_bytes':len(prefix),'runner_after':{'bytes':len(adapted_runner),'sha256':sha(adapted_runner)},
                     'runner_body_preserved':adapted_runner[len(prefix):]==runner_before,
                     'complete_jadx_source_before':{'bytes':len(source_bytes),'sha256':sha(source_bytes)},
                     'complete_jadx_source_after':{'bytes':len(copied_source.read_bytes()),'sha256':sha(copied_source.read_bytes())}}
            if not adapter['runner_body_preserved'] or adapter['complete_jadx_source_before']!=adapter['complete_jadx_source_after']:
                raise ValueError(f'{key}: source adapter changed Runner body or complete JADX source')
            ckey=f'jadx-compile-{key}-{leg}'
            crow,_,_=run.command(ckey,javac_argv(leg,copied_source,copied_runner,classes),cwd=REPO,java_home=JDKS[leg]['home'],allow_nonzero=True)
            result={'case':key,'jdk':leg,'compile_exit':crow['exit'],'source':file_pin(copied_source),'runner_source':file_pin(copied_runner),'runner_adapter':adapter,'classes_dir':str(classes)}
            if crow['exit']==0:
                rrow,rout,rerr=run.command(f'jadx-runtime-{key}-{leg}',java_argv(leg,classes,adapter['main_class_qualified']),cwd=REPO,java_home=JDKS[leg]['home'],allow_nonzero=True)
                result.update({'runtime_exit':rrow['exit'],'runtime_stdout_sha256':sha(rout),'runtime_stderr_sha256':sha(rerr),'runtime_matches_oracle':rrow['exit']==0 and rout==(out/'oracles'/(key+'.stdout.raw')).read_bytes()})
            execution.setdefault('jadx_results',[]).append(result); atomic_json(out/'execution.json',execution)

    # Render fresh complete Jarde source for both evidence settings for every input class.
    # Then compile each full source with the matching Runner. Direct-return additionally runs if compilation succeeds.
    for case,src,runner_src,classfile in cases:
        key=case['id']; leg=case['source_jdk']
        for evidence in ('default','all'):
            label=f'jarde-render-{key}-{evidence}'
            argv=[str(CLI),'class-source','--input',str(classfile),'--class',case['class_name'],'--policy','single-class','--release','8','--format','json']
            if evidence=='all': argv += ['--evidence','all']
            row,outbytes,err=run.command(label,argv,cwd=REPO,java_home=JDKS[leg]['home'],allow_nonzero=True)
            result={'case':key,'evidence':evidence,'render_exit':row['exit'],'render_stdout_sha256':sha(outbytes),'render_stderr_sha256':sha(err)}
            if row['exit']==0:
                try:
                    doc=json.loads(outbytes.decode('utf-8'))
                    srcbytes=doc.get('text')
                    if not isinstance(srcbytes,str): raise ValueError('JSON text field is not a string')
                    sourcefile=case_path(out,f'jarde-source/{key}/{evidence}')/(case['class_name']+'.java')
                    sourcefile.write_text(srcbytes,encoding='utf-8',newline='')
                    result['source']=file_pin(sourcefile)
                    result['quality_summary']={'outcome':doc.get('outcome'),'methods':[{'name':bytes(m['item']['identity']['name']).decode('utf-8'),'outcome':m.get('outcome'),'text_bytes':len(m.get('text') or '')} for m in doc.get('methods',[])]}
                    dest=case_path(out,f'jarde-build/{key}/{evidence}'); classes=dest/'classes'; classes.mkdir()
                    copied_runner=dest/runner_src.name; shutil.copyfile(runner_src,copied_runner)
                    copied_source=dest/sourcefile.name; shutil.copyfile(sourcefile,copied_source)
                    cr,cso,cse=run.command(f'jarde-compile-{key}-{evidence}',javac_argv(leg,copied_source,copied_runner,classes),cwd=REPO,java_home=JDKS[leg]['home'],allow_nonzero=True)
                    result.update({'compile_exit':cr['exit'],'compile_stdout_sha256':sha(cso),'compile_stderr_sha256':sha(cse),'compile_interpreted_as_runtime':False})
                    if case['fixture']=='direct-return' and cr['exit']==0:
                        rr,rout,rerr=run.command(f'jarde-runtime-{key}-{evidence}',java_argv(leg,classes,case['main_class']),cwd=REPO,java_home=JDKS[leg]['home'],allow_nonzero=True)
                        result.update({'runtime_exit':rr['exit'],'runtime_matches_original_oracle':rr['exit']==0 and rout==(out/'oracles'/(key+'.stdout.raw')).read_bytes(),'runtime_stdout_sha256':sha(rout),'runtime_stderr_sha256':sha(rerr)})
                except Exception as exc:
                    result['capture_error']=f'{type(exc).__name__}: {exc}'
            execution.setdefault('jarde_results',[]).append(result); atomic_json(out/'execution.json',execution)
    execution['status']='completed_capture'
    execution['interpretation']='JADX direct results are actual complete-source compile/runtime observations. Jarde compile nonzero is a source-compile failure only; no Jarde runtime is inferred from it. Direct-return Jarde runtime is attempted only after whole-source compile succeeds.'
    execution['command_count']=len(run.records); execution['commands']=run.records
    atomic_json(out/'execution.json',execution)
    print(str(out/'execution.json'))

if __name__=='__main__':
    try: main()
    except Exception as e:
        print(f'runner stopped after preserving execution.json and raw streams: {type(e).__name__}: {e}',file=sys.stderr)
        raise
