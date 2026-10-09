#!/usr/bin/env python3
"""Isolate the historical 23-family field runner and record its exact inputs."""
import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUNNER = ROOT/'openspec/changes/recover-class-scope-constructor-parameters/results/field-regression/replay.py'
EVIDENCE = ROOT/'openspec/evidence/generic-holder-write-boundaries'
ROOT_PROBES = ROOT/'openspec/changes/prove-generic-field-write-source-types/results'
SOURCE_AUDIT = ROOT/'openspec/changes/prove-generic-field-write-source-types/results/acceptance-manifest.json'
BASELINE_SHA = '3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70'
JDKS = {
  'javac8': Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
  'javac23': Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'),
}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--baseline',type=Path,default=Path('/tmp/jarde-raw-receiver-final-v3-cli'))
    parser.add_argument('--candidate',type=Path,required=True)
    parser.add_argument('--candidate-sha256',required=True)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args()
    baseline=args.baseline.resolve(); candidate=args.candidate.resolve(); out=args.out.resolve()
    if out.exists(): raise SystemExit('refusing to overwrite field-23 output: '+str(out))
    if not baseline.is_file() or sha(baseline)!=BASELINE_SHA: raise SystemExit('accepted baseline CLI hash mismatch')
    if not candidate.is_file() or sha(candidate)!=args.candidate_sha256: raise SystemExit('candidate CLI hash mismatch')
    out.mkdir(parents=True)
    (out/'empty-classpath').mkdir(); (out/'empty-sourcepath').mkdir()
    adapter_copy=out/'field-23-replay-adapter-executed.py'; adapter_copy.write_bytes(Path(__file__).resolve().read_bytes())
    runner_copy=out/'replay-executed.py'; runner_copy.write_bytes(RUNNER.read_bytes())
    sources=[(p.stem,p) for p in sorted(EVIDENCE.glob('*/source/*.java'))]
    sources += [(p.stem,p) for p in sorted((ROOT_PROBES/'root-anchors').glob('*.java'))]
    sources += [(p.parent.name,p) for p in sorted((ROOT_PROBES/'root-probes').glob('*/*.java'))]
    if len(sources)!=23 or len({name for name,_ in sources})!=23:
        raise SystemExit(f'expected 23 unique source families, got {len(sources)}')
    source_by_path={str(p.resolve()):{'family':name,'path':str(p),'sha256':sha(p)} for name,p in sources}
    source_by_name={name:source_by_path[str(path.resolve())] for name,path in sources}
    source_audit=json.loads(SOURCE_AUDIT.read_text())
    historical_sources={name for name,path in sources if path.is_relative_to(EVIDENCE)}
    historical_jars={}
    for name in sorted(historical_sources):
        src=next(path for family,path in sources if family==name)
        audit_key=str(src.relative_to(ROOT))
        if source_audit['source_sha256'].get(audit_key)!=sha(src):
            raise SystemExit('historical source SHA differs from accepted input manifest: '+audit_key)
        for leg in JDKS:
            jar=EVIDENCE/leg/name/(name+'.jar')
            if not jar.is_file(): raise SystemExit('historical JAR missing for mapped source: '+str(jar))
            with zipfile.ZipFile(jar) as archive:
                entries=sorted(x for x in archive.namelist() if x.endswith('.class'))
                if entries!=[name+'.class']:
                    raise SystemExit(f'historical JAR/source mapping is not one-to-one: {jar}: {entries}')
            historical_jars[(leg,name)]={'path':jar,'sha256':sha(jar),'source':source_by_name[name],
              'class_sha256':{name+'.class':hashlib.sha256(zipfile.ZipFile(jar).read(name+'.class')).hexdigest()}}
    if len(historical_jars)!=28 or len(historical_sources)!=14:
        raise SystemExit(f'expected 14 families with two historical JDK jars, got {len(historical_sources)}/{len(historical_jars)}')
    actual_commands=[]; input_javac=[]; temporary_jars=[]; reuse_events=[]
    logs=out/'adapter-command-logs'; logs.mkdir()
    def record_command(label, actual, result):
        idx=len(actual_commands); stdout=logs/f'{idx:04d}-{label}.stdout'; stderr=logs/f'{idx:04d}-{label}.stderr'
        stdout.write_bytes(result.stdout or b''); stderr.write_bytes(result.stderr or b'')
        actual_commands.append({'label':label,'argv':actual,'exit':result.returncode,
          'stdout':str(stdout),'stdout_sha256':sha(stdout),'stderr':str(stderr),'stderr_sha256':sha(stderr)})
    namespace={'__name__':'field23_replay','__file__':str(RUNNER)}
    exec(compile(runner_copy.read_bytes(),str(runner_copy),'exec'),namespace)
    if namespace['main'].__globals__ is not namespace or namespace['run'].__globals__ is not namespace:
        raise SystemExit('saved runner functions do not share the execution namespace')
    namespace['RESULTS']=out
    real_run=subprocess.run
    real_module_run=namespace['run']
    def isolated_module_run(argv, stem):
        actual=[str(x) for x in argv]
        source_matches=[source_by_path[x] for x in actual if x in source_by_path]
        if source_matches and source_matches[0]['family'] in historical_sources and str(stem).endswith('original-compile'):
            source=source_matches[0]; family=source['family']
            leg='javac8' if str(actual[0]).startswith(str(JDKS['javac8'])) else 'javac23'
            hist=historical_jars[(leg,family)]; classes=Path(actual[actual.index('-d')+1]); classes.mkdir(parents=True,exist_ok=True)
            extraction=[str(JDKS[leg]/'bin/jar'),'xf',str(hist['path'])]
            extracted=real_run(extraction,cwd=classes,capture_output=True)
            record_command('reuse-historical-jar-extract',extraction,extracted)
            if extracted.returncode: raise SystemExit('failed to extract historical original classes: '+family)
            reuse_events.append({'family':family,'leg':leg,'source':source,'jar':str(hist['path']),
              'jar_sha256':hist['sha256'],'class_sha256':hist['class_sha256'],'original_classes_dir':str(classes),
              'note':'original classes loaded from historical JAR; javac input compilation skipped'})
            result=subprocess.CompletedProcess(argv,0,b'',b'')
            base=Path(stem); base.with_suffix('.stdout').write_bytes(result.stdout); base.with_suffix('.stderr').write_bytes(result.stderr)
            input_javac.append({'source':source,'reused_jar':str(hist['path']),'reused_jar_sha256':hist['sha256'],
              'javac_skipped_for_historical_input_reuse':True})
            return result
        if source_matches and str(stem).endswith('original-compile'):
            input_javac.append({'source':source_matches[0],'javac_skipped_for_historical_input_reuse':False,
              'argv':[str(x) for x in argv]})
        return real_module_run(argv,stem)
    namespace['run']=isolated_module_run
    saved_hook=namespace['run']
    saved_main=namespace['main']
    sentinel=object()
    namespace['run']=lambda *unused: sentinel
    exec("def _adapter_main_stub(): return RESULTS, run('stub', 'stub')",namespace)
    namespace['main']=namespace['_adapter_main_stub']
    probe_out,probe_hook=namespace['main']()
    namespace['main']=saved_main
    del namespace['_adapter_main_stub']
    if Path(probe_out)!=out or probe_hook is not sentinel:
        raise SystemExit('saved runner main namespace does not observe output/hook overrides')
    namespace['run']=saved_hook
    def traced_run(argv,*pos,**kwargs):
        actual=[str(x) for x in argv]
        if Path(actual[0]).name=='javac':
            if '-classpath' not in actual and '-cp' not in actual:
                actual=[actual[0],'-classpath',str(out/'empty-classpath'),'-sourcepath',str(out/'empty-sourcepath'),*actual[1:]]
            found=[source_by_path[x] for x in actual if x in source_by_path]
            if found: input_javac.append({'source':found[0],'argv':actual})
        jar_info=None
        if Path(actual[0]).name=='jar' and len(actual)>4 and actual[1]=='cf' and '-C' in actual:
            jar_path=Path(actual[2]); family=jar_path.stem
            leg='javac8' if str(actual[0]).startswith(str(JDKS['javac8'])) else 'javac23'
            if (leg,family) in historical_jars:
                hist=historical_jars[(leg,family)]
                shutil.copyfile(hist['path'],jar_path)
                jar_info={'family':family,'leg':leg,'temporary_path':str(jar_path),'jar_sha256':sha(jar_path),
                  'class_sha256':hist['class_sha256'],'source':hist['source'],'kind':'historical-jar-byte-copy'}
                result=subprocess.CompletedProcess(actual,0,b'',b'')
                record_command('reuse-historical-jar-copy',actual,result)
                temporary_jars.append(jar_info)
                return result
            classes=Path(actual[actual.index('-C')+1])
            class_hashes={str(p.relative_to(classes)):sha(p) for p in sorted(classes.rglob('*.class'))}
            jar_info={'family':family,'leg':leg,'temporary_path':str(jar_path),'class_sha256':class_hashes,'kind':'source-rebuilt-temp-jar'}
        result=real_run(actual,*pos,**kwargs)
        record_command(Path(actual[0]).name,actual,result)
        if jar_info is not None:
            jar_info['jar_sha256']=sha(jar_path) if jar_path.is_file() else None
            jar_info['source']=source_by_name.get(jar_path.stem)
            temporary_jars.append(jar_info)
        return result
    subprocess.run=traced_run
    failure=None
    try:
        sys.argv=[str(RUNNER),'--baseline',str(baseline),'--candidate',str(candidate)]
        namespace['main']()
    except BaseException as exc:
        failure=repr(exc)
    finally:
        subprocess.run=real_run
    metadata={'adapter_snapshot':str(adapter_copy),'adapter_snapshot_sha256':sha(adapter_copy),
      'runner_snapshot':str(runner_copy),'runner_snapshot_sha256':sha(runner_copy),
      'historical_runner_path':str(RUNNER),'baseline_cli':str(baseline),'baseline_cli_sha256':sha(baseline),
      'candidate_cli':str(candidate),'candidate_cli_sha256':sha(candidate),'output_label':'candidate',
      'family_count':len(sources),'source_families':[{'family':name,'path':str(path),'sha256':sha(path)} for name,path in sources],
      'source_audit_manifest':str(SOURCE_AUDIT),'source_audit_manifest_sha256':sha(SOURCE_AUDIT),
      'historical_input_jars':[{'leg':leg,'family':name,'path':str(data['path']),'sha256':data['sha256'],
        'source':data['source'],'class_sha256':data['class_sha256']} for (leg,name),data in sorted(historical_jars.items())],
      'historical_jar_reuse_events':reuse_events,
      'source_only_rebuilt_families':[{'family':name,'path':str(path),'sha256':sha(path)} for name,path in sources if name not in historical_sources],
      'jdk_homes':{name:str(path) for name,path in JDKS.items()},'javac_with_empty_paths':input_javac,
      'temporary_original_jars':temporary_jars,'actual_commands':actual_commands,'runner_error':failure}
    (out/'run-metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')
    if failure: raise SystemExit('field runner failed; see run-metadata.json: '+failure)
    print(json.dumps({'out':str(out),'families':len(sources),'baseline_sha256':sha(baseline),
      'candidate_sha256':sha(candidate),'temporary_input_jars':len(temporary_jars)},indent=2))


if __name__=='__main__':
    main()
