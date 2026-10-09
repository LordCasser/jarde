#!/usr/bin/env python3
"""Independent four-leg evidence for nested generic call-result arguments."""
import argparse
import hashlib
import json
import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
JDKS = {
    'corretto8': (Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'), ['-source', '8', '-target', '8']),
    'openjdk23': (Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'), ['--release', '8']),
}
DEBUGS = {'debug': ['-g'], 'nodebug': ['-g:none']}
JADX_DEFAULT = Path('/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx')
SOURCE = '''public class NestedCallArgument<T> {
    public T first(T x) { return x; }
    public T second(T x) { return x; }
    public T relay(T x) { return second(first(x)); }
}
'''
PROBE = '''import java.lang.reflect.*;
public class NestedCallProbe {
  static int failures;
  static void check(boolean ok, String label) {
    System.out.println("check."+label+"="+ok);
    if (!ok) failures++;
  }
  public static void main(String[] args) throws Exception {
    Class<?> c=Class.forName(args[0]);
    TypeVariable<?>[] vars=c.getTypeParameters();
    check(vars.length==1, "one-class-formal");
    TypeVariable<?> t=vars[0];
    check(t.getGenericDeclaration()==c, "class-formal-owner");
    for(String name:new String[]{"first","second","relay"}) {
      Method m=c.getDeclaredMethod(name,Object.class);
      check(m.getGenericReturnType().equals(t), name+"-return-is-class-T");
      check(m.getGenericParameterTypes()[0].equals(t), name+"-parameter-is-class-T");
      check(((TypeVariable<?>)m.getGenericReturnType()).getGenericDeclaration()==c,
            name+"-return-declaration-is-class");
      System.out.println("method="+name+";return="+m.getGenericReturnType().getTypeName()
          +";parameter="+m.getGenericParameterTypes()[0].getTypeName());
    }
    Object receiver=c.getConstructor().newInstance();
    Object marker=new Object();
    Object returned=c.getDeclaredMethod("relay",Object.class).invoke(receiver,marker);
    check(returned==marker, "nested-result-marker-identity");
    System.out.println("behavior=nested-result-marker-identity");
    System.out.println("probe.failures="+failures);
    if(failures!=0) System.exit(1);
  }
}
'''


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(commands, label, argv, cwd, outdir, stem):
    argv = [str(x) for x in argv]
    proc = subprocess.run(argv, cwd=cwd, capture_output=True)
    stdout, stderr = outdir/(stem+'.stdout'), outdir/(stem+'.stderr')
    stdout.write_bytes(proc.stdout)
    stderr.write_bytes(proc.stderr)
    commands.append({'label':label,'argv':argv,'cwd':str(cwd),'exit':proc.returncode,
                     'stdout':str(stdout),'stdout_sha256':sha(stdout),
                     'stderr':str(stderr),'stderr_sha256':sha(stderr)})
    return proc


def class_name(files, expected):
    for path in files:
        package = None
        for line in path.read_text(errors='replace').splitlines():
            line=line.strip()
            if line.startswith('package ') and line.endswith(';'):
                package=line[8:-1]
        if expected in path.stem:
            return (package+'.' if package else '')+expected
    return expected


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--cli',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    parser.add_argument('--jadx',type=Path,default=JADX_DEFAULT)
    args=parser.parse_args()
    cli=args.cli.resolve(); jadx=args.jadx.resolve(); out=args.out.resolve()
    if out.exists(): raise SystemExit('refusing to overwrite extension output: '+str(out))
    if not cli.is_file() or not jadx.is_file(): raise SystemExit('CLI or JADX launcher missing')
    out.mkdir(parents=True)
    runner_copy=out/'replay-executed.py'; shutil.copyfile(Path(__file__).resolve(),runner_copy)
    source=out/'input'/'NestedCallArgument.java'; source.parent.mkdir(parents=True)
    source.write_text(SOURCE)
    source_sha=sha(source)
    probe_template=out/'NestedCallProbe.java'; probe_template.write_text(PROBE)
    commands=[]; rows=[]; jars=[]
    libdir=jadx.parent.parent/'lib'; libs=sorted(libdir.glob('*.jar'))
    if len(libs)!=57: raise SystemExit('expected 57 built JADX libraries')
    for leg,(home,release) in JDKS.items():
      for debug,debug_args in DEBUGS.items():
        key=leg+'/'+debug; area=out/leg/debug; area.mkdir(parents=True)
        copied=area/'NestedCallArgument.java'; shutil.copyfile(source,copied)
        probe=area/'NestedCallProbe.java'; shutil.copyfile(probe_template,probe)
        cp=area/'empty-classpath'; cp.mkdir(); sp=area/'empty-sourcepath'; sp.mkdir()
        javac=home/'bin/javac'; java=home/'bin/java'; jar=home/'bin/jar'; javap=home/'bin/javap'
        with tempfile.TemporaryDirectory(prefix='nested-call-input-') as td:
          tmp=Path(td); input_classes=tmp/'classes'; input_classes.mkdir()
          original_cmd=run(commands,'input-javac',[javac,*release,*debug_args,'-classpath',cp,'-sourcepath',sp,'-d',input_classes,copied],area,area,'input-javac')
          if original_cmd.returncode: raise SystemExit('original extension source failed to compile: '+key)
          class_hashes={str(p.relative_to(input_classes)):sha(p) for p in sorted(input_classes.rglob('*.class'))}
          jarfile=area/'NestedCallArgument.input.jar'
          jar_cmd=run(commands,'input-jar',[jar,'cf',jarfile,'-C',input_classes,'.'],area,area,'input-jar')
          if jar_cmd.returncode: raise SystemExit('input jar failed: '+key)
        jadx_area=area/'jadx'; jadx_area.mkdir(); jadx_out=jadx_area/'full-output'; jadx_out.mkdir()
        jadx_cmd=run(commands,'jadx',[jadx,'--no-res','-d',jadx_out,jarfile],area,jadx_area,'jadx')
        jadx_files=sorted(jadx_out.rglob('*.java'))
        if jadx_cmd.returncode or not jadx_files: raise SystemExit('JADX failed to decompile extension input: '+key)
        jarde_area=area/'jarde'; jarde_area.mkdir()
        jarde_cmd=run(commands,'jarde-class-source',[cli,'class-source','--input',jarfile,'--class','NestedCallArgument','--policy','plain-jar','--release','8','--format','text'],area,jarde_area,'class-source')
        jarde_source=jarde_area/'NestedCallArgument.java'; jarde_source.write_bytes(jarde_cmd.stdout)
        class_header=any(line.strip().startswith(('public class NestedCallArgument','class NestedCallArgument')) for line in jarde_cmd.stdout.decode(errors='replace').splitlines())
        legrow={'leg':leg,'debug':debug,'input_source_sha256':sha(copied),'probe_source_sha256':sha(probe),
          'input_jar':str(jarfile),'input_jar_sha256':sha(jarfile),'input_class_sha256':class_hashes,
          'jarde_exit':jarde_cmd.returncode,'jarde_source_sha256':sha(jarde_source),'jarde_nonempty_class_header':class_header,
          'jadx_exit':jadx_cmd.returncode,'jadx_sources':[{'path':str(p.relative_to(area)),'sha256':sha(p)} for p in jadx_files],
          'flavors':{}}
        if jarde_cmd.returncode not in (0,4) or not class_header: raise SystemExit('Jarde source/header failure: '+key)
        flavors={'original':[copied],'jadx':jadx_files,'baseline':[jarde_source]}
        names={f:class_name(files,'NestedCallArgument') for f,files in flavors.items()}
        for flavor,files in flavors.items():
          fdir=area/'compile-sources'/flavor; fdir.mkdir(parents=True)
          copied_files=[]
          for i,path in enumerate(files):
            target=fdir/path.name
            if target.exists(): target=fdir/(str(i)+'-'+path.name)
            shutil.copyfile(path,target);copied_files.append(target)
          with tempfile.TemporaryDirectory(prefix='nested-call-classes-') as td:
            classes=Path(td)/'classes';classes.mkdir()
            compile_cmd=run(commands,flavor+'-javac',[javac,*release,*debug_args,'-classpath',cp,'-sourcepath',sp,'-d',classes,*copied_files,probe],area,fdir,flavor+'-javac')
            result={'sources':[{'path':str(p.relative_to(area)),'sha256':sha(p)} for p in copied_files],
              'compile_exit':compile_cmd.returncode,'compiled_classes_sha256':{str(p.relative_to(classes)):sha(p) for p in sorted(classes.rglob('*.class'))}}
            if compile_cmd.returncode==0:
              runtime=run(commands,flavor+'-probe',[java,'-Xverify:all','-cp',classes,'NestedCallProbe',names[flavor]],area,fdir,flavor+'-probe')
              result['probe_exit']=runtime.returncode;result['probe_stdout']=runtime.stdout.decode(errors='replace')
              result['behavior_marker']='behavior=nested-result-marker-identity' in result['probe_stdout']
              result['generic_owner_checks']=sum('=true' in line for line in result['probe_stdout'].splitlines() if line.startswith('check.'))
              result['probe_failures']=sum(1 for line in result['probe_stdout'].splitlines() if line.startswith('check.') and line.endswith('=false'))
              result['reflection_sha256']=hashlib.sha256('\n'.join(line for line in result['probe_stdout'].splitlines() if line.startswith('method=')).encode()).hexdigest()
            legrow['flavors'][flavor]=result
          jv=run(commands,flavor+'-javap',[javap,'-p','-s','-v','-classpath',jarfile,'NestedCallArgument'],area,area,flavor+'-input-javap') if flavor=='original' else None
        rows.append(legrow)
        jars.append({'leg':leg,'debug':debug,'path':str(jarfile.relative_to(out)),'sha256':sha(jarfile),'class_sha256':class_hashes,'source_sha256':source_sha})
    manifest={'scope':'Independent nested call-argument supplement; not part of the frozen 140-input matrix.',
      'fixture':'NestedCallArgument<T>{first(T),second(T),relay(T x){return second(first(x));}}',
      'source_sha256':source_sha,'probe_template_sha256':sha(probe_template),'cli':str(cli),'cli_sha256':sha(cli),
      'jadx':str(jadx),'jadx_sha256':sha(jadx),'jadx_lib_count':len(libs),
      'jadx_libs':[{'path':str(p),'sha256':sha(p)} for p in libs],
      'runner_source':str(runner_copy),'runner_source_sha256':sha(runner_copy),
      'jdk_legs':{name:{'home':str(home),'javac_sha256':sha(home/'bin/javac'),'java_sha256':sha(home/'bin/java'),'javap_sha256':sha(home/'bin/javap'),'release':release}
                  for name,(home,release) in JDKS.items()},
      'debug_modes':DEBUGS,'isolated_search_paths':'fresh empty classpath and sourcepath for all javac commands; runtime classpath contains only each flavor compiled classes',
      'inputs':jars,'commands':commands,'cases':rows}
    manifest['files']=[{'path':str(p.relative_to(out)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(out.rglob('*')) if p.is_file() and p.name!='manifest.json']
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({'out':str(out),'inputs':len(rows),'cli_sha256':sha(cli),'runner_sha256':sha(runner_copy),
      'compile':{f:sum(c['flavors'][f]['compile_exit']==0 for c in rows) for f in ('original','jadx','baseline')},
      'probes':{f:sum(c['flavors'][f].get('probe_exit')==0 for c in rows) for f in ('original','jadx','baseline')}},indent=2))


if __name__=='__main__':
    main()
