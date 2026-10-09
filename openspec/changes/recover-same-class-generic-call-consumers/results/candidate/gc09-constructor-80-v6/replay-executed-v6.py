#!/usr/bin/env python3
"""Replay frozen class-scope generic constructor fixtures with Jarde CLIs."""
import argparse, difflib, json, re, shutil, subprocess, tempfile
from pathlib import Path

ROOT=Path(__file__).resolve().parent
JDKS={'corretto8':Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
      'openjdk23':Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home')}
MANIFEST=json.loads((ROOT/'manifest.json').read_text())
POS={name for name,case in MANIFEST.items() if case['kind']=='positive'}
PARTIAL_POS={name for name,case in MANIFEST.items() if case['kind']=='partial_positive'}
CASES=sorted(MANIFEST)

def scope_match(left,right,prefix):
    a=[x for x in left if x.startswith(prefix)]
    b=[x for x in right if x.startswith(prefix)]
    return bool(a) and a==b

def peer_partial_reflection(lines):
    one=[x for x in lines if x.startswith('REFLECT|ctor[') and '.formalCount=1;' in x]
    two=[x for x in lines if x.startswith('REFLECT|ctor[') and '.formalCount=2;' in x]
    if len(one)!=1 or len(two)!=1: return False
    one_id=one[0].split(']',1)[0]+']'
    two_id=two[0].split(']',1)[0]+']'
    return (one_id+'.param[0]=java.lang.Object;binder=non-variable' in lines and
            two_id+'.param[0]=T;binder=class' in lines and
            two_id+'.param[1]=boolean;binder=non-variable' in lines and
            'REFLECT|field[v]=java.lang.Object;binder=non-variable' in lines)

def run(argv,base):
    p=subprocess.run([str(x) for x in argv],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    base.with_suffix('.stdout').write_bytes(p.stdout)
    base.with_suffix('.stderr').write_bytes(p.stderr)
    base.with_suffix('.exit').write_text(str(p.returncode)+'\n')
    return p

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--baseline')
    ap.add_argument('--candidate')
    ap.add_argument('--out',default=str(ROOT/'runs'))
    ap.add_argument('--only',nargs='*',help='optional class-family subset')
    a=ap.parse_args()
    clis={k:v for k,v in [('baseline',a.baseline),('candidate',a.candidate)] if v}
    if not clis: ap.error('provide --baseline and/or --candidate')
    selected=set(a.only or CASES)
    unknown=selected-set(CASES)
    if unknown: ap.error('unknown class family: '+', '.join(sorted(unknown)))
    outroot=Path(a.out)
    summary=[]
    jadx_summary=[]
    for leg,jdk in JDKS.items():
      flags=['-source','8','-target','8'] if leg=='corretto8' else ['--release','8']
      for mode in ('debug','nodebug'):
        modeflags=flags+(['-g'] if mode=='debug' else ['-g:none'])
        for name in sorted(selected):
          frozen=ROOT/'frozen'/leg/mode/name
          original=frozen/'classes'; jar=frozen/(name+'.jar')
          caseout=outroot/leg/mode/name
          caseout.mkdir(parents=True,exist_ok=True)
          assert jar.is_file() and original.joinpath(name+'.class').is_file(), (leg,mode,name,'missing frozen original')
          with tempfile.TemporaryDirectory(prefix='jarde-ctor-replay-') as temp_name:
            temp=Path(temp_name); driver=temp/'driver'; driver.mkdir()
            dp=run([jdk/'bin/javac',*modeflags,'-d',driver,ROOT/'ConstructorDriver.java'],caseout/'driver-javac')
            if dp.returncode: raise RuntimeError((name,leg,mode,'external Driver compile failed',dp.stderr.decode(errors='replace')))
            original_run=run([jdk/'bin/java','-Xverify:all','-cp',f'{original}:{driver}','ConstructorDriver',name],caseout/'original-run')
            if original_run.returncode: raise RuntimeError((name,leg,mode,'original run failed',original_run.stderr.decode(errors='replace')))
            original_lines=original_run.stdout.decode().splitlines()
            behavior_original=[x for x in original_lines if x.startswith('BEHAVIOR|')]
            reflect_original=[x for x in original_lines if x.startswith('REFLECT|')]
            # Recompile complete JADX class source with the same external Driver.
            jadx_files=list((frozen/'jadx'/'sources').rglob(name+'.java'))
            jr={'leg':leg,'debug':mode,'class':name,'source_count':len(jadx_files)}
            jsrc=''
            if len(jadx_files)==1:
              jsrc=jadx_files[0].read_text()
              jsrc=re.sub(r'^\s*package\s+defpackage\s*;\s*\n','',jsrc,count=1)
              javadir=caseout/'jadx-source'
              if javadir.exists(): shutil.rmtree(javadir)
              javadir.mkdir()
              (javadir/(name+'.java')).write_text(jsrc)
              jr['source_header']=bool(jsrc.strip()) and re.search(r'\bclass\s+'+re.escape(name)+r'\b',jsrc) is not None
              jclasses=temp/'jadx-classes'; jclasses.mkdir()
              jcp=run([jdk/'bin/javac',*modeflags,'-d',jclasses,javadir/(name+'.java')],caseout/'jadx-javac')
              jr['compile_exit']=jcp.returncode
              if jcp.returncode==0:
                jrun=run([jdk/'bin/java','-Xverify:all','-cp',f'{jclasses}:{driver}','ConstructorDriver',name],caseout/'jadx-run')
                jlines=jrun.stdout.decode(errors='replace').splitlines()
                jr['run_exit']=jrun.returncode
                jr['behavior_match']=jrun.returncode==0 and [x for x in jlines if x.startswith('BEHAVIOR|')]==behavior_original
                jr['constructor_reflection_match']=jrun.returncode==0 and scope_match(jlines,reflect_original,'REFLECT|ctor[')
                jr['field_reflection_match']=jrun.returncode==0 and scope_match(jlines,reflect_original,'REFLECT|field[')
                jr['class_reflection_match']=jrun.returncode==0 and scope_match(jlines,reflect_original,'REFLECT|class')
                jr['reflection_match']=jr['constructor_reflection_match'] and jr['field_reflection_match'] and jr['class_reflection_match']
              else:
                jr['run_exit']=None; jr['behavior_match']=False; jr['reflection_match']=False; jr['constructor_reflection_match']=False; jr['field_reflection_match']=False; jr['class_reflection_match']=False
            else:
              jr.update({'source_header':False,'compile_exit':None,'run_exit':None,'behavior_match':False,'reflection_match':False,'constructor_reflection_match':False,'field_reflection_match':False,'class_reflection_match':False})
            jadx_summary.append(jr)
            for version,cli in clis.items():
              vout=caseout/version
              if vout.exists(): shutil.rmtree(vout)
              vout.mkdir(parents=True)
              p=run([cli,'class-source','--input',jar,'--class',name,'--policy','plain-jar','--format','json','--evidence','all','--release','8'],vout/'class-source')
              rec={'leg':leg,'debug':mode,'class':name,'version':version,'cli_exit':p.returncode}
              report=None; source=''
              try:
                report=json.loads(p.stdout)
                source=report.get('text','')
                (vout/'report.json').write_bytes(p.stdout)
                (vout/(name+'.java')).write_text(source)
              except (json.JSONDecodeError,UnicodeDecodeError) as e:
                rec['json_error']=repr(e)
              rec['source_header']=bool(source.strip()) and source.startswith('// jarde: presentation of `'+name+'`') and re.search(r'\bclass\s+'+re.escape(name)+r'\b',source) is not None
              rec['cli_status_acceptable']=p.returncode in (0,4)
              if rec['source_header']:
                gen=temp/(version+'-classes'); gen.mkdir()
                cp=run([jdk/'bin/javac',*modeflags,'-d',gen,vout/(name+'.java')],vout/'javac')
                rec['compile_exit']=cp.returncode
                if cp.returncode==0:
                  # This classpath intentionally contains generated classes and the external Driver only.
                  rp=run([jdk/'bin/java','-Xverify:all','-cp',f'{gen}:{driver}','ConstructorDriver',name],vout/'run')
                  rec['run_exit']=rp.returncode
                  rec['run_output']=rp.stdout.decode(errors='replace')
                  output=rp.stdout.decode(errors='replace').splitlines()
                  behavior=[x for x in output if x.startswith('BEHAVIOR|')]
                  reflect=[x for x in output if x.startswith('REFLECT|')]
                  rec['behavior_match']=rp.returncode==0 and behavior==behavior_original
                  rec['constructor_reflection_match']=rp.returncode==0 and scope_match(reflect,reflect_original,'REFLECT|ctor[')
                  rec['field_reflection_match']=rp.returncode==0 and scope_match(reflect,reflect_original,'REFLECT|field[')
                  rec['class_reflection_match']=rp.returncode==0 and scope_match(reflect,reflect_original,'REFLECT|class')
                  rec['reflection_match']=rec['constructor_reflection_match'] and rec['field_reflection_match'] and rec['class_reflection_match']
                  if name in PARTIAL_POS:
                    rec['partial_constructor_and_field_match']=rp.returncode==0 and peer_partial_reflection(reflect)
                  (vout/'behavior.diff').write_text(''.join(difflib.unified_diff([x+'\n' for x in behavior_original],[x+'\n' for x in behavior],fromfile='original',tofile=version)))
                  (vout/'reflection.diff').write_text(''.join(difflib.unified_diff([x+'\n' for x in reflect_original],[x+'\n' for x in reflect],fromfile='original',tofile=version)))
                else:
                  rec['run_exit']=None; rec['behavior_match']=False; rec['reflection_match']=False; rec['constructor_reflection_match']=False; rec['field_reflection_match']=False; rec['class_reflection_match']=False
              else:
                rec['compile_exit']=None; rec['run_exit']=None; rec['behavior_match']=False; rec['reflection_match']=False; rec['constructor_reflection_match']=False; rec['field_reflection_match']=False; rec['class_reflection_match']=False
              summary.append(rec)
    outroot.mkdir(parents=True,exist_ok=True)
    (outroot/'summary.json').write_text(json.dumps({'jarde':summary,'jadx':jadx_summary},indent=2)+'\n')
    bad_cli=[x for x in summary if not x['cli_status_acceptable'] or not x['source_header']]
    if bad_cli: raise AssertionError(f'CLI status/header failures recorded in {outroot}/summary.json: {bad_cli}')
    if 'baseline' in clis and 'candidate' in clis:
      by_key={(x['leg'],x['debug'],x['class'],x['version']):x for x in summary}
      regressions=[]
      for baseline in (x for x in summary if x['version']=='baseline' and x.get('compile_exit')==0):
        candidate=by_key[(baseline['leg'],baseline['debug'],baseline['class'],'candidate')]
        if candidate.get('compile_exit')!=0 or candidate.get('run_exit')!=0 or not candidate.get('behavior_match'):
          regressions.append({'class':baseline['class'],'leg':baseline['leg'],'debug':baseline['debug'],
                              'baseline_compile':baseline.get('compile_exit'),
                              'candidate_compile':candidate.get('compile_exit'),
                              'candidate_run':candidate.get('run_exit'),
                              'candidate_behavior_match':candidate.get('behavior_match')})
      if regressions:
        raise AssertionError(f'baseline-compilable families regressed; details in {outroot}/summary.json: {regressions}')
    for version in clis:
      group=[x for x in summary if x['version']==version]
      print(version,'source_header',sum(bool(x.get('source_header')) for x in group),'/',len(group),
            'compile',sum(x.get('compile_exit')==0 for x in group),'/',len(group),
            'runtime',sum(x.get('run_exit')==0 for x in group),'/',len(group),
            'behavior-match',sum(x.get('behavior_match') for x in group),'/',len(group),
            'reflection-match',sum(x.get('reflection_match') for x in group),'/',len(group),
            'ctor-reflection-match',sum(x.get('constructor_reflection_match') for x in group),'/',len(group))
      if version=='candidate':
        required=[x for x in group if x['class'] in POS]
        failures=[x for x in required if x.get('compile_exit')!=0 or x.get('run_exit')!=0 or not x.get('behavior_match') or not x.get('reflection_match')]
        partial=[x for x in group if x['class'] in PARTIAL_POS]
        failures += [x for x in partial if x.get('compile_exit')!=0 or x.get('run_exit')!=0 or not x.get('behavior_match') or not x.get('partial_constructor_and_field_match')]
        cross=[x for x in group if x['class']=='CrossHold']
        failures += [x for x in cross if x.get('compile_exit')!=0 or x.get('run_exit')!=0 or not x.get('behavior_match') or not x.get('constructor_reflection_match') or 'REFLECT|field[v]=java.lang.Object;binder=non-variable' not in (x.get('run_output',''))]
        if failures: raise AssertionError(f'required positive failures recorded in {outroot}/summary.json: {failures}')
if __name__=='__main__': main()
