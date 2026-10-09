#!/usr/bin/env python3
"""Independent whole-class acceptance; no Rust builds and no original jar on output classpaths."""
import argparse
import difflib
import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[5]
RESULTS = Path(__file__).resolve().parent
EVIDENCE = ROOT / 'openspec/evidence/generic-holder-write-boundaries'
INPUT_RESULTS = ROOT / 'openspec/changes/prove-generic-field-write-source-types/results'
JDKS = {
    'javac8': Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
    'javac23': Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'),
}
FULL_REFLECTION = {'TypedSetter', 'NullSetter', 'RawListField', 'SCGA', 'SCGBCompat', 'RawBoundWriter'}
PROBE_DRIVER = '''import java.lang.reflect.*;
import java.util.*;
public class ProbeDriver {
  public static void main(String[] a) throws Exception {
    Class<?> c=Class.forName(a[0]); Object o=c.getConstructor().newInstance();
    for(Method m:c.getDeclaredMethods()) if(m.getName().equals("put")) {
      Class<?>[] ps=m.getParameterTypes(); Object[] xs=new Object[ps.length];
      for(int i=0;i<ps.length;i++) {
        if(ps[i]==long.class) xs[i]=17L;
        else if(ps[i]==double.class) xs[i]=2.5;
        else if(ps[i].isArray()) xs[i]=new List[]{Collections.singletonList("array")};
        else if(List.class.isAssignableFrom(ps[i])) xs[i]=Collections.singletonList("list");
        else xs[i]="arg"+i;
      }
      m.invoke(o,xs);
    }
    Object v=c.getField("v").get(o);
    System.out.println("value="+(v instanceof Object[] ? Arrays.deepToString((Object[])v) : v));
    System.out.println("classvars="+c.getTypeParameters().length);
    System.out.println("fieldGenericType="+c.getField("v").getGenericType().getTypeName());
  }
}
'''

def run(argv, stem):
    result = subprocess.run([str(x) for x in argv], capture_output=True)
    stem.with_suffix('.stdout').write_bytes(result.stdout)
    stem.with_suffix('.stderr').write_bytes(result.stderr)
    return result

def semantic(text):
    return [line for line in text.splitlines()
            if 'GenericType=' not in line and not line.startswith('method=')]

def reflect_args(name):
    ctor = 'ctor' if name in {'Hold', 'ObjectHold'} else 'default'
    methods = [] if ctor == 'ctor' else ['put']
    if name == 'MixedSetter': methods = ['putT', 'putObject']
    if name == 'NullSetter': methods = ['put', 'clear']
    if name == 'SCGA': methods = ['put', 'main']
    if name in {'SCGB', 'SCGBCompat'}: methods = ['main']
    return [name, ctor, 'default'] + methods

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--candidate', required=True)
    parser.add_argument('--baseline', default='/tmp/jarde-generic-baseline-cli')
    args = parser.parse_args()
    summary = []
    for leg, jdk in JDKS.items():
        with tempfile.TemporaryDirectory(prefix='jarde-field-root-') as temporary:
            temporary = Path(temporary)
            drivers = temporary / 'drivers'; drivers.mkdir()
            reflect = temporary / 'ReflectDriver.java'
            reflect_source = (EVIDENCE/'ReflectDriver.java').read_text()
            reflect_source = reflect_source.replace(
                'if (parameter.isArray())',
                'if (parameter==boolean.class) return Boolean.TRUE;\n'
                '    if (Map.class.isAssignableFrom(parameter)) return new HashMap<String,String>(Collections.singletonMap("raw","value"));\n'
                '    if (parameter.isArray())')
            reflect_source = reflect_source.replace(
                'else m.invoke(o,argument(m.getParameterTypes()[0],method,c));',
                'else { Class<?>[] ps=m.getParameterTypes(); Object[] xs=new Object[ps.length]; '
                'for(int k=0;k<ps.length;k++) xs[k]=argument(ps[k],method,c); m.invoke(o,xs); }')
            reflect.write_text(reflect_source)
            probe = temporary / 'ProbeDriver.java'; probe.write_text(PROBE_DRIVER)
            driver_compile = subprocess.run([str(jdk/'bin/javac'), '-g:none', '-d', str(drivers),
                                             str(reflect), str(probe)], capture_output=True)
            assert driver_compile.returncode == 0, driver_compile.stderr
            families = [(p.stem, p, False) for p in sorted(EVIDENCE.glob('*/source/*.java'))]
            families += [(p.stem, p, False) for p in sorted((INPUT_RESULTS/'root-anchors').glob('*.java'))]
            families += [(p.parent.name, p, True) for p in sorted((INPUT_RESULTS/'root-probes').glob('*/*.java'))]
            for name, source, is_probe in families:
                out = RESULTS / 'root' / leg / name; out.mkdir(parents=True, exist_ok=True)
                original = temporary / name / 'original'; original.mkdir(parents=True)
                flags = ['-source', '8', '-target', '8'] if leg == 'javac8' else ['--release', '8']
                compiled = run([jdk/'bin/javac', *flags, '-g:none', '-d', original, source], out/'original-compile')
                assert compiled.returncode == 0, (name, compiled.stderr)
                jar = temporary / name / (name+'.jar')
                subprocess.run([str(jdk/'bin/jar'), 'cf', str(jar), '-C', str(original), '.'], check=True)
                driver = 'ProbeDriver' if is_probe else 'ReflectDriver'
                inputs = [name] if is_probe else reflect_args(name)
                original_run = run([jdk/'bin/java', '-Xverify:all', '-cp', str(original)+':'+str(drivers),
                                    driver, *inputs], out/'original-run')
                assert original_run.returncode == 0, (name, original_run.stderr)
                reports = {}
                for version, cli in [('baseline', args.baseline), ('candidate', args.candidate)]:
                    result = run([cli, 'class-source', '--input', jar, '--class', name, '--policy',
                                  'plain-jar', '--format', 'json', '--evidence', 'all', '--release', '8'], out/version)
                    assert result.returncode in {0, 4}, (name, version, result.stderr)
                    report = json.loads(result.stdout); reports[version] = report
                    assert report['text'].startswith('// jarde: presentation of `'+name+'`')
                    assert 'public class '+name in report['text']
                    (out/(version+'.json')).write_bytes(result.stdout)
                    sources = temporary / name / version; sources.mkdir()
                    java = sources/(name+'.java'); java.write_text(report['text'])
                    (out/(version+'.java')).write_text(report['text'])
                    classes = sources/'classes'; classes.mkdir()
                    compiled = run([jdk/'bin/javac', *flags, '-g:none', '-d', classes, java], out/(version+'-compile'))
                    entry = {'leg':leg,'class':name,'version':version,'compile':compiled.returncode,
                             'reflection_scope': 'field_only' if is_probe else 'all_declared_parameters_and_field'}
                    if compiled.returncode == 0:
                        execution = run([jdk/'bin/java', '-Xverify:all', '-cp', str(classes)+':'+str(drivers),
                                         driver, *inputs], out/(version+'-run'))
                        entry['run'] = execution.returncode
                        entry['behavior_match'] = semantic(execution.stdout.decode()) == semantic(original_run.stdout.decode())
                        entry['reflection_match'] = execution.stdout == original_run.stdout
                        diff = difflib.unified_diff(original_run.stdout.decode().splitlines(True), execution.stdout.decode().splitlines(True), fromfile='original', tofile=version)
                        (out/(version+'-reflection.diff')).write_text(''.join(diff))
                        if version == 'candidate':
                            assert execution.returncode == 0 and entry['behavior_match'], entry
                            if name in FULL_REFLECTION:
                                assert entry['reflection_match'], entry
                    elif version == 'candidate':
                        assert name == 'SCGB' and 'jarde_refused_body' in report['text'], entry
                    summary.append(entry)
                for key in ['class', 'stages']:
                    assert reports['baseline'][key] == reports['candidate'][key], (name,key)
                assert [x['item'] for x in reports['baseline']['fields']] == [x['item'] for x in reports['candidate']['fields']]
                assert [x['item'] for x in reports['baseline']['methods']] == [x['item'] for x in reports['candidate']['methods']]
    (RESULTS/'root'/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    for leg in JDKS:
        group = [x for x in summary if x['leg']==leg]
        print(leg, {version: sum(x['compile']==0 for x in group if x['version']==version)
                    for version in ['baseline','candidate']}, 'of',len(group)//2)

if __name__ == '__main__':
    main()
