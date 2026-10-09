#!/usr/bin/env python3
"""Replay a candidate CLI against the four already-frozen nested-call jars."""
import argparse
import hashlib
import json
import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
FROZEN_DEFAULT = ROOT/'openspec/changes/recover-same-class-generic-call-consumers/results/extension/nested-call-argument-v1'
JDKS = {
    'corretto8': (Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'), ['-source','8','-target','8']),
    'openjdk23': (Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'), ['--release','8']),
}
DEBUGS = {'debug':['-g'],'nodebug':['-g:none']}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(commands, label, argv, cwd, logdir, stem):
    argv=[str(x) for x in argv]
    proc=subprocess.run(argv,cwd=cwd,capture_output=True)
    stdout=logdir/(stem+'.stdout'); stderr=logdir/(stem+'.stderr')
    stdout.write_bytes(proc.stdout); stderr.write_bytes(proc.stderr)
    commands.append({'label':label,'argv':argv,'cwd':str(cwd),'exit':proc.returncode,
      'stdout':str(stdout),'stdout_sha256':sha(stdout),'stderr':str(stderr),'stderr_sha256':sha(stderr)})
    return proc


def class_name(source, expected):
    package=None
    for line in source.read_text(errors='replace').splitlines():
        line=line.strip()
        if line.startswith('package ') and line.endswith(';'):
            package=line[8:-1]
    return (package+'.' if package else '')+expected


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--frozen-inputs',type=Path,default=FROZEN_DEFAULT)
    parser.add_argument('--cli',type=Path,required=True)
    parser.add_argument('--cli-sha256',required=True)
    parser.add_argument('--label',choices=('candidate',),required=True)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args()
    frozen=args.frozen_inputs.resolve(); cli=args.cli.resolve(); out=args.out.resolve()
    source_manifest=frozen/'manifest.json'
    if not source_manifest.is_file(): raise SystemExit('frozen nested-call input manifest missing')
    frozen_manifest=json.loads(source_manifest.read_text())
    if not cli.is_file() or sha(cli)!=args.cli_sha256: raise SystemExit('candidate CLI SHA-256 mismatch')
    if out.exists(): raise SystemExit('refusing to overwrite candidate result directory: '+str(out))
    if frozen.resolve()==out or frozen.resolve() in out.parents: raise SystemExit('candidate output must be isolated from frozen inputs')
    for record in frozen_manifest['files']:
        path=frozen/record['path']
        if not path.is_file() or path.stat().st_size!=record['bytes'] or sha(path)!=record['sha256']:
            raise SystemExit('frozen nested-call evidence hash mismatch: '+record['path'])
    expected={(leg,debug) for leg in JDKS for debug in DEBUGS}
    rows={(case['leg'],case['debug']):case for case in frozen_manifest['cases']}
    if set(rows)!=expected or len(rows)!=4: raise SystemExit('expected exactly four frozen input legs')
    out.mkdir(parents=True)
    runner_copy=out/'replay-executed.py'; shutil.copyfile(Path(__file__).resolve(),runner_copy)
    frozen_manifest_sha=sha(source_manifest); commands=[]; results=[]
    for leg,(home,release) in JDKS.items():
      for debug,debug_args in DEBUGS.items():
        case=rows[(leg,debug)]; area=out/leg/debug; area.mkdir(parents=True)
        input_case=frozen/leg/debug
        jar=input_case/'NestedCallArgument.input.jar'
        if sha(jar)!=case['input_jar_sha256']: raise SystemExit('frozen input jar hash differs from manifest')
        probe_source=input_case/'NestedCallProbe.java'
        if sha(probe_source)!=case['probe_source_sha256']: raise SystemExit('frozen Probe source hash differs from manifest')
        probe=area/'NestedCallProbe.java'; shutil.copyfile(probe_source,probe)
        cp=area/'empty-classpath'; cp.mkdir(); sp=area/'empty-sourcepath'; sp.mkdir()
        cli_area=area/'candidate-source'; cli_area.mkdir()
        javac=home/'bin/javac'; java=home/'bin/java'
        cli_result=run(commands,'candidate-class-source',
          [cli,'class-source','--input',jar,'--class','NestedCallArgument','--policy','plain-jar','--release','8','--format','text'],
          area,cli_area,'class-source')
        emitted=cli_area/'NestedCallArgument.java'; emitted.write_bytes(cli_result.stdout)
        header=any(line.strip().startswith(('public class NestedCallArgument','class NestedCallArgument'))
                   for line in cli_result.stdout.decode(errors='replace').splitlines())
        row={'leg':leg,'debug':debug,'input_jar':str(jar),'input_jar_sha256':sha(jar),
          'input_source_sha256':case['input_source_sha256'],'probe_source_sha256':sha(probe),
          'cli_label':args.label,'cli_sha256':sha(cli),'class_source_exit':cli_result.returncode,
          'candidate_source_sha256':sha(emitted),'candidate_nonempty_class_header':header}
        with tempfile.TemporaryDirectory(prefix='nested-candidate-classes-') as td:
          classes=Path(td)/'classes'; classes.mkdir()
          compile_result=run(commands,'candidate-javac',
            [javac,*release,*debug_args,'-classpath',cp,'-sourcepath',sp,'-d',classes,emitted,probe],
            area,cli_area,'candidate-javac')
          row['compile_exit']=compile_result.returncode
          row['compiled_classes_sha256']={str(p.relative_to(classes)):sha(p) for p in sorted(classes.rglob('*.class'))}
          if compile_result.returncode==0:
            name=class_name(emitted,'NestedCallArgument')
            probe_result=run(commands,'candidate-probe',
              [java,'-Xverify:all','-cp',classes,'NestedCallProbe',name],area,cli_area,'candidate-probe')
            stdout=probe_result.stdout.decode(errors='replace')
            original_probe=input_case/'compile-sources/original/original-probe.stdout'
            checks=[line for line in stdout.splitlines() if line.startswith('check.')]
            row.update({'probe_exit':probe_result.returncode,'probe_stdout':stdout,
              'probe_failures':[line for line in checks if line.endswith('=false')],
              'generic_declaration_and_marker_check_count':sum(line.endswith('=true') for line in checks),
              'behavior_marker':'behavior=nested-result-marker-identity' in stdout,
              'probe_matches_frozen_original':probe_result.stdout==original_probe.read_bytes(),
              'frozen_original_probe_sha256':sha(original_probe)})
        if row['class_source_exit'] not in (0,4): row['class_source_status']='cli-error'
        elif not header: row['class_source_status']='missing-class-header'
        else: row['class_source_status']='source-emitted'
        results.append(row)
    manifest={'scope':'Candidate replay against the four frozen nested-call input jars; no input jar is rebuilt.',
      'cli_label':args.label,'candidate_cli':str(cli),'candidate_cli_sha256':sha(cli),
      'frozen_inputs':str(frozen),'frozen_manifest_sha256':frozen_manifest_sha,
      'runner_source':str(runner_copy),'runner_source_sha256':sha(runner_copy),
      'jdk_legs':{leg:{'home':str(home),'javac_sha256':sha(home/'bin/javac'),'java_sha256':sha(home/'bin/java'),'release':release}
                  for leg,(home,release) in JDKS.items()},
      'debug_modes':DEBUGS,'runtime_classpath':'candidate compiled classes only; -Xverify:all',
      'javac_search_paths':'fresh empty classpath and sourcepath directories',
      'input_jars':[{'leg':leg,'debug':debug,'path':str((frozen/leg/debug/'NestedCallArgument.input.jar')),
                     'sha256':rows[(leg,debug)]['input_jar_sha256']} for leg,debug in sorted(expected)],
      'commands':commands,'cases':results}
    manifest['files']=[{'path':str(p.relative_to(out)),'bytes':p.stat().st_size,'sha256':sha(p)}
                       for p in sorted(out.rglob('*')) if p.is_file() and p.name!='manifest.json']
    (out/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    totals={'inputs':len(results),'source_emitted':sum(r['class_source_status']=='source-emitted' for r in results),
      'full_class_compile':sum(r['compile_exit']==0 for r in results),
      'probe_pass':sum(r.get('probe_exit')==0 and not r.get('probe_failures') for r in results),
      'generic_checks':sum(r.get('generic_declaration_and_marker_check_count',0) for r in results),
      'behavior_markers':sum(bool(r.get('behavior_marker')) for r in results),
      'matches_frozen_original':sum(bool(r.get('probe_matches_frozen_original')) for r in results)}
    (out/'summary.json').write_text(json.dumps({'cli_label':args.label,'cli_sha256':sha(cli),'totals':totals},indent=2)+'\n')
    manifest['files']=[{'path':str(p.relative_to(out)),'bytes':p.stat().st_size,'sha256':sha(p)}
                       for p in sorted(out.rglob('*')) if p.is_file() and p.name!='manifest.json']
    (out/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'out':str(out),'cli_label':args.label,'cli_sha256':sha(cli),**totals},indent=2))


if __name__=='__main__':
    main()
