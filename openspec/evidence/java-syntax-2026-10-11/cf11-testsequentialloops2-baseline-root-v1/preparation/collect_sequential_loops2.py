#!/usr/bin/env python3
"""Collect root-frozen CF11 SequentialLoops2 observations. Preparation only; run after root freezes inputs/tools."""
from __future__ import annotations
import argparse, datetime, hashlib, importlib.util, json, os, re, shutil, sys, zipfile
from pathlib import Path
sys.dont_write_bytecode = True
ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
MIN_FREE, MAX_TARGET = 5 * 1024**3, 1024**3
STRIP_ENV = ('JAVA_HOME','JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')

def sha(data): return hashlib.sha256(data).hexdigest()
def pin(path):
    data=Path(path).read_bytes(); return {'path':str(Path(path).resolve()),'bytes':len(data),'sha256':sha(data)}
def write_json(path,value):
    path=Path(path); path.parent.mkdir(parents=True,exist_ok=True); path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def import_pinned(path, expected, name):
    if pin(path)['sha256'] != expected: raise RuntimeError(name+' pin mismatch')
    spec=importlib.util.spec_from_file_location('cf11_controls_'+name,path); module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module
class Recorder:
    def __init__(self,out,guard,execution): self.out,self.guard,self.execution=out,guard,execution
    def run(self,label,argv,home):
        index=len(self.execution['commands'])+1; argv=[str(x) for x in argv]
        row={'index':index,'label':label,'argv':argv,'cwd':str(ROOT),'java_home':str(home),'state':'started','started_at':datetime.datetime.now(datetime.timezone.utc).isoformat()}
        self.execution['commands'].append(row); self.execution['status']='running'; write_json(self.out/'execution.json',self.execution)
        env={k:v for k,v in os.environ.items() if k not in STRIP_ENV}; env['JAVA_HOME']=str(home); env['PATH']=str(home/'bin')+os.pathsep+env.get('PATH','')
        try:
            row.update(self.guard.run_command(index,argv,env)); row.update(label=label,java_home=str(home),state='observed')
            stdout=Path(row['streams']['stdout']['path']).read_bytes(); stderr=Path(row['streams']['stderr']['path']).read_bytes()
        except BaseException as exc:
            row.update(state='collector-error',error_type=type(exc).__name__,error=str(exc)); self.execution.update(status='command-error',last_error={'label':label,'type':type(exc).__name__,'message':str(exc)}); write_json(self.out/'execution.json',self.execution); raise
        write_json(self.out/'execution.json',self.execution); return row,stdout,stderr

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--inputs-dir',type=Path,required=True); p.add_argument('--inputs-manifest',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True); p.add_argument('--cli',type=Path,required=True); p.add_argument('--cli-sha256',required=True)
    p.add_argument('--build-receipt',type=Path,required=True); p.add_argument('--build-receipt-sha256',required=True)
    p.add_argument('--receipt-cli-object',default='candidate_cli'); p.add_argument('--receipt-status-key',default='status'); p.add_argument('--receipt-accepted-value',default='passed_candidate_build_only'); p.add_argument('--receipt-mode-key',default='mode')
    for name in ('jdk-manifest','jadx','jadx-jar','guard','scanner','classfile-parser'):
        p.add_argument('--'+name,type=Path,required=True); p.add_argument('--'+name+'-sha256',required=True)
    a=p.parse_args()
    for k,v in vars(a).items():
        if isinstance(v,Path): setattr(a,k,v.resolve())
    out=a.out
    if out.exists(): raise SystemExit('refuse overwrite: '+str(out))
    if not a.cli.is_file() or not os.access(a.cli,os.X_OK) or (a.cli.stat().st_mode&0o777)!=0o555: raise SystemExit('frozen CLI missing, non-executable, or not mode 0555; no command run')
    for path,digest,name in ((a.cli,a.cli_sha256,'CLI'),(a.build_receipt,a.build_receipt_sha256,'build receipt')):
        if pin(path)['sha256']!=digest: raise SystemExit(name+' pin mismatch; no command run')
    receipt=json.loads(a.build_receipt.read_text(encoding='utf-8')); binding=receipt
    for part in a.receipt_cli_object.split('.'):
        if not isinstance(binding,dict) or part not in binding: raise SystemExit('receipt binding path absent; no command run')
        binding=binding[part]
    if (receipt.get(a.receipt_status_key)!=a.receipt_accepted_value or a.receipt_accepted_value!='passed_candidate_build_only'
        or not isinstance(binding,dict) or binding.get('path')!=str(a.cli) or binding.get('bytes')!=a.cli.stat().st_size
        or binding.get('sha256')!=a.cli_sha256 or binding.get(a.receipt_mode_key)!='0o555'):
        raise SystemExit('receipt does not bind a 0555 passed_candidate_build_only CLI; no command run')
    manifest_pin=pin(a.inputs_manifest); manifest=json.loads(a.inputs_manifest.read_text(encoding='utf-8'))
    if manifest.get('package')!='cf11' or manifest.get('source_level')!='Java 8': raise SystemExit('inputs manifest package/source level mismatch; no command run')
    classes=manifest.get('target_classes'); runner=manifest.get('runner')
    def entry_path(item):
        rel=Path(item['source']); path=(a.inputs_dir/rel).resolve()
        if a.inputs_dir not in path.parents: raise SystemExit('input escapes inputs-dir; no command run')
        if pin(path)['bytes']!=item['bytes'] or pin(path)['sha256']!=item['sha256']: raise SystemExit('input manifest pin mismatch: '+str(path))
        return path
    targets=[x.get('class') for x in classes] if isinstance(classes,list) else []
    if not targets or len(set(targets))!=len(targets) or not all(isinstance(x,str) and re.fullmatch(r'cf11\.[A-Za-z_$][A-Za-z0-9_$]*',x) for x in targets) or 'cf11.Runner' in targets or not isinstance(runner,dict) or runner.get('class')!='cf11.Runner': raise SystemExit('manifest must bind unique cf11 target classes plus Runner; no command run')
    sources={x['class']:entry_path(x) for x in classes}; runner_path=entry_path(runner)
    if any(Path(x['source']).name!=x['class'].split('.')[-1]+'.java' for x in classes) or Path(runner['source']).name!='Runner.java': raise SystemExit('manifest source filename/class mismatch; no command run')
    guard=import_pinned(a.guard,a.guard_sha256,'guard'); scanner=import_pinned(a.scanner,a.scanner_sha256,'scanner')
    import_pinned(a.classfile_parser,a.classfile_parser_sha256,'classfile_parser')
    tool_pins=[(a.jdk_manifest,a.jdk_manifest_sha256,'JDK manifest'),(a.jadx,a.jadx_sha256,'JADX'),(a.jadx_jar,a.jadx_jar_sha256,'JADX jar'),(a.scanner,a.scanner_sha256,'scanner'),(a.classfile_parser,a.classfile_parser_sha256,'class parser')]
    for path,digest,name in tool_pins:
        if pin(path)['sha256']!=digest: raise SystemExit(name+' pin mismatch; no command run')
    jm=json.loads(a.jdk_manifest.read_text(encoding='utf-8'))
    if pin(a.jdk_manifest)['sha256']!=a.jdk_manifest_sha256: raise SystemExit('JDK manifest pin mismatch; no command run')
    legs={}
    for e in jm.get('legs',[]):
        tools=e.get('jdk_tools',{})
        if {'java','javac','javap'} <= tools.keys():
            facts={n:{'path':Path(v['path']).resolve(),'sha256':v['sha256']} for n,v in tools.items()}
            if all(pin(v['path'])['sha256']==v['sha256'] for v in facts.values()): legs[e['leg']]={'tools':{n:v['path'] for n,v in facts.items()},'pins':facts,'home':facts['java']['path'].parent.parent}
    if set(legs)!={'javac8','javac23'}: raise SystemExit('pinned JDK8/JDK23 legs required; no command run')
    out.mkdir(parents=True)
    for name in ('raw','inputs','legs','jadx','jarde'): (out/name).mkdir()
    guard.OUT=out/'raw'; guard.ROOT=ROOT; guard.expected_test_summaries=lambda index,stdout:None; guard.ENV_VALUES={}
    guard.target_bytes=lambda:scanner.target_bytes(ROOT)
    guard.command_stream=lambda path:{'path':str(Path(path).resolve()),'bytes':Path(path).stat().st_size,'sha256':sha(Path(path).read_bytes())}
    copied={}
    for cls,path in sources.items():
        dest=out/'inputs'/path.name; shutil.copyfile(path,dest); copied[cls]=dest
    runner=out/'inputs'/'Runner.java'; shutil.copyfile(runner_path,runner)
    execution={'schema':'cf11-target-full-source-capture-v2','status':'preflight_complete','package':'cf11','classes':targets,'runner_class':'cf11.Runner','inputs_manifest':manifest_pin,'inputs_manifest_path':str(a.inputs_manifest),'input_entries':manifest,'commands':[],'original_compile_rows':[],'javap':[],'jadx':{},'jadx_compile_rows':[],'jarde_profiles':[],'jarde_compile_rows':[],'build_receipt':pin(a.build_receipt),'build_receipt_cli_binding':binding,'cli':pin(a.cli),'cli_mode':oct(a.cli.stat().st_mode&0o777),'resource_guard':{'minimum_free_bytes':MIN_FREE,'maximum_target_bytes':MAX_TARGET,'poll_seconds':1,'peak_target_sampled':True},'collector_config':{k:(str(v) if isinstance(v,Path) else v) for k,v in vars(a).items()}}
    tracked=[Path(__file__),a.inputs_manifest,a.cli,a.build_receipt,a.jdk_manifest,a.jadx,a.jadx_jar,a.guard,a.scanner,a.classfile_parser,*[x['path'] for leg in legs.values() for x in leg['pins'].values()],*sources.values(),runner_path]
    execution['source_tool_pins_before']=[pin(x) for x in tracked]; write_json(out/'execution.json',execution); rec=Recorder(out,guard,execution)
    class_paths=sorted([c.replace('.','/')+'.class' for c in targets]+['cf11/Runner.class'])
    def compile_case(label,srcs,jdk,oracle=None):
        leg=legs[jdk]; work=out/'legs'/label; classes_dir=work/'classes'; empty=work/'empty-paths'; classes_dir.mkdir(parents=True); empty.mkdir()
        flags=['-source','8','-target','8','-Xlint:-options'] if jdk=='javac8' else ['--release','8']
        comp,_,_=rec.run(label+'-compile',[leg['tools']['javac'],*flags,'-proc:none','-g','-classpath',empty,'-sourcepath',empty,'-d',classes_dir,*srcs],leg['home'])
        run=None
        if comp.get('exit_code')==0 and comp.get('guard_stop') is None: run,_,_=rec.run(label+'-run',[leg['tools']['java'],'-Xverify:all','-cp',classes_dir,'cf11.Runner'],leg['home'])
        files=sorted(p for p in classes_dir.rglob('*.class') if p.is_file()); paths=sorted(p.relative_to(classes_dir).as_posix() for p in files)
        runtime=None
        if run is not None and oracle is not None:
            rr=next(x for x in rec.execution['commands'] if x['label']==run['label']); runtime={'same_jdk_original':run.get('exit_code')==oracle['exit_code'] and Path(rr['streams']['stdout']['path']).read_bytes()==oracle['stdout'] and Path(rr['streams']['stderr']['path']).read_bytes()==oracle['stderr']}
        return {'label':label,'compile_command':comp['label'],'compile_exit':comp.get('exit_code'),'run_command':run['label'] if run else None,'run_exit':run.get('exit_code') if run else None,'source_pins':[pin(x) for x in srcs],'class_paths':paths,'class_set_exact':(paths==class_paths) if comp.get('exit_code')==0 else None,'classes':[pin(x) for x in files],'runtime_comparison':runtime}
    def javap(jdk,tag,cls):
        cf=out/'legs'/(jdk+'-'+tag)/'classes'/ (cls.replace('.','/')+'.class')
        if not cf.is_file(): return
        leg=legs[jdk]; row,stdout,stderr=rec.run(jdk+'-'+tag+'-javap-'+cls.split('.')[-1],[leg['tools']['javap'],'-p','-c','-s','-v','-classpath',cf.parent.parent,cls],leg['home'])
        path=out/'legs'/(jdk+'-'+tag)/('javap-'+cls.split('.')[-1]+'.txt'); path.write_bytes(stdout)
        execution['javap'].append({'class':cls,'jdk':jdk,'command':row['label'],'stdout_pin':pin(path),'stderr_sha256':sha(stderr)})
    originals={}; oracles={}
    for jdk in ('javac8','javac23'):
        row=compile_case(jdk+'-original',list(copied.values())+[runner],jdk); execution['original_compile_rows'].append(row)
        for cls in targets:
            cf=out/'legs'/(jdk+'-original')/'classes'/(cls.replace('.','/')+'.class')
            if cf.is_file(): originals[(jdk,cls)]=cf; javap(jdk,'original',cls)
        if row['run_command']:
            rr=next(x for x in execution['commands'] if x['label']==row['run_command']); oracles[jdk]={'exit_code':rr['exit_code'],'stdout':Path(rr['streams']['stdout']['path']).read_bytes(),'stderr':Path(rr['streams']['stderr']['path']).read_bytes()}
        write_json(out/'execution.json',execution)
    # JADX receives precisely the Java 23 original target class files.
    jar=out/'inputs'/'sequential-loops2.jar'
    if all(('javac23',c) in originals for c in targets):
        with zipfile.ZipFile(jar,'w',zipfile.ZIP_STORED) as z:
            for cls in targets:
                info=zipfile.ZipInfo(cls.replace('.','/')+'.class',date_time=(1980,1,1,0,0,0)); info.compress_type=zipfile.ZIP_STORED; z.writestr(info,originals[('javac23',cls)].read_bytes())
    ver,vo,ve=rec.run('jadx-version',[a.jadx,'--version'],legs['javac23']['home'])
    jadx_profiles={}
    for profile in ('default','none'):
        dest=out/'jadx'/profile; argv=[a.jadx,'--no-res','--config','none','--threads-count','1']
        if profile=='none': argv+=['--rename-flags','none']
        command,_,_=rec.run('jadx-'+profile,[*argv,'-d',dest,jar],legs['javac23']['home'])
        files=sorted(dest.rglob('*.java')) if dest.exists() else []; pkgs=[]
        for f in files:
            m=re.search(r'(?m)^\s*package\s+([\w.]+)\s*;',f.read_text(encoding='utf-8')); pkgs.append(m.group(1) if m else None)
        expected_names=sorted(c.split('.')[-1]+'.java' for c in targets); exact=sorted(x.name for x in files)==expected_names and pkgs==['cf11']*len(targets)
        jadx_profiles[profile]={'command':command['label'],'exit_code':command.get('exit_code'),'sources':[pin(x) for x in files],'names':[x.name for x in files],'packages':pkgs,'exact_target_sources':exact}
        if exact:
            copied_dir=out/'inputs'/('jadx-'+profile); copied_dir.mkdir()
            for f in files: shutil.copyfile(f,copied_dir/f.name)
            for jdk in ('javac8','javac23'):
                srcs=[copied_dir/f'{c.split(".")[-1]}.java' for c in targets]+[runner]
                execution['jadx_compile_rows'].append(compile_case(jdk+'-jadx-'+profile,srcs,jdk,oracles.get(jdk)))
        write_json(out/'execution.json',execution)
    execution['jadx']={'version_command':ver['label'],'version_exit':ver.get('exit_code'),'version_stdout_sha256':sha(vo),'version_stderr_sha256':sha(ve),'jar_pin':pin(jar) if jar.exists() else None,'jar_entries':targets,'profiles':jadx_profiles}
    # Render each original class separately; source bodies are preserved byte-for-byte.
    reports={}
    for jdk in ('javac8','javac23'):
        for cls in targets:
            original=originals.get((jdk,cls))
            if original is None: continue
            for mode in ('default','all'):
                label=f'{jdk}-jarde-{mode}-{cls.split(".")[-1]}'
                argv=[a.cli,'class-source','--input',original,'--class',cls,'--policy','single-class','--release','8','--format','json']
                if mode=='all': argv+=['--evidence','all']
                row,raw,err=rec.run(label+'-render',argv,legs[jdk]['home'])
                item={'label':label,'class':cls,'jdk':jdk,'mode':mode,'render_command':row['label'],'render_exit':row.get('exit_code'),'render_stdout_sha256':sha(raw),'render_stderr_sha256':sha(err),'compile_run':None}
                if row.get('exit_code')==0:
                    try:
                        doc=json.loads(raw.decode('utf-8')); folder=out/'jarde'/label; folder.mkdir(parents=True)
                        report=folder/'report.json'; write_json(report,doc); source=folder/(cls.split('.')[-1]+'.java'); source.write_text(doc['text'],encoding='utf-8')
                        item.update(report_pin=pin(report),source_pin=pin(source),report_execution=doc.get('execution'))
                        reports[(jdk,mode,cls)]={'doc':doc,'source':source,'item':item}
                    except (ValueError,TypeError,KeyError,UnicodeError) as exc: item['parse_error']={'type':type(exc).__name__,'message':str(exc)}
                execution['jarde_profiles'].append(item); write_json(out/'execution.json',execution)
    for jdk in ('javac8','javac23'):
        for mode in ('default','all'):
            generated=[reports.get((jdk,mode,c)) for c in targets]
            if not all(generated): continue
            label=f'{jdk}-jarde-{mode}-combined'; srcs=[x['source'] for x in generated]+[runner]
            combined=compile_case(label,srcs,jdk,oracles.get(jdk)); execution['jarde_compile_rows'].append(combined); write_json(out/'execution.json',execution)
            if combined['compile_exit']!=0:
                for index,target in enumerate(generated):
                    isolate_srcs=[target['source']]+[copied[c] for c in targets if c!=targets[index]]+[runner]
                    row=compile_case(f'{jdk}-jarde-{mode}-isolated-{targets[index].split(".")[-1]}',isolate_srcs,jdk,oracles.get(jdk)); execution['jarde_compile_rows'].append(row); write_json(out/'execution.json',execution)
    execution['source_tool_pins_after']=[pin(x) for x in tracked]
    execution['source_tool_inputs_unchanged']=execution['source_tool_pins_before']==execution['source_tool_pins_after']
    execution['actual_command_count']=len(execution['commands']); execution['status']='observed' if execution['source_tool_inputs_unchanged'] else 'input_pin_changed'
    write_json(out/'execution.json',execution)
    return 0
if __name__=='__main__': raise SystemExit(main())
