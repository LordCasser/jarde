#!/usr/bin/env python3
"""Preserve and independently replay the 18 source-rebuilt original field inputs."""
import difflib
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path(__file__).resolve().parent / 'complete-run'
META = ROOT / 'openspec/changes/recover-same-class-generic-call-consumers/results/candidate/gc09-field-23-v9/adapter-run/run-metadata.json'
BASELINE = Path('/private/tmp/jarde-raw-receiver-final-v3-cli')
BASELINE_SHA = '3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70'
CANDIDATE = Path('/private/tmp/jarde-generic-calls-candidate-v9-cli')
CANDIDATE_SHA = '5bb2fcfdcf958839f0b1e060a2e55114510c6115cb225ec3279d7d7adf95e006'
JADX = Path('/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx')
EVIDENCE = ROOT / 'openspec/evidence/generic-holder-write-boundaries'
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


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def hash_classes(classes):
    return {str(path.relative_to(classes)): sha(path)
            for path in sorted(classes.rglob('*.class'))}


def run(commands, label, argv, cwd, output_dir, stem):
    argv = [str(value) for value in argv]
    result = subprocess.run(argv, cwd=cwd, capture_output=True)
    stdout = output_dir / f'{stem}.stdout'
    stderr = output_dir / f'{stem}.stderr'
    stdout.write_bytes(result.stdout or b'')
    stderr.write_bytes(result.stderr or b'')
    commands.append({
        'label': label, 'argv': argv, 'cwd': str(cwd), 'exit': result.returncode,
        'stdout': str(stdout), 'stdout_sha256': sha(stdout),
        'stderr': str(stderr), 'stderr_sha256': sha(stderr),
    })
    return result


def class_name(sources, expected):
    for path in sources:
        package = None
        for line in path.read_text(errors='replace').splitlines():
            line = line.strip()
            if line.startswith('package ') and line.endswith(';'):
                package = line[8:-1]
        if expected in path.stem:
            return (package + '.' if package else '') + expected
    return expected


def semantic(output):
    return [line for line in output.splitlines()
            if 'GenericType=' not in line and not line.startswith('method=')]


def reflect_args(name):
    ctor = 'ctor' if name in {'Hold', 'ObjectHold'} else 'default'
    methods = [] if ctor == 'ctor' else ['put']
    if name == 'MixedSetter':
        methods = ['putT', 'putObject']
    if name == 'NullSetter':
        methods = ['put', 'clear']
    if name == 'SCGA':
        methods = ['put', 'main']
    if name in {'SCGB', 'SCGBCompat'}:
        methods = ['main']
    return [name, ctor, 'default'] + methods


def main():
    if OUT.exists() and any(OUT.iterdir()):
        raise SystemExit('refusing non-empty field18-msrv-final output directory')
    OUT.mkdir(parents=True, exist_ok=True)
    if sha(BASELINE) != BASELINE_SHA or sha(CANDIDATE) != CANDIDATE_SHA:
        raise SystemExit('fixed CLI identity mismatch')
    metadata = json.loads(META.read_text())
    targets = [row for row in metadata['temporary_original_jars']
               if row.get('kind') == 'source-rebuilt-temp-jar']
    if len(targets) != 18:
        raise SystemExit(f'expected exactly 18 source-rebuilt original JARs, got {len(targets)}')

    shutil.copyfile(Path(__file__).resolve(), OUT / 'replay-executed.py')
    inputs = OUT / 'permanent-original-inputs'
    inputs.mkdir()
    results = OUT / 'cases'
    results.mkdir()
    empty_cp_root = OUT / 'empty-classpath'
    empty_sp_root = OUT / 'empty-sourcepath'
    empty_cp_root.mkdir(); empty_sp_root.mkdir()
    commands = []
    rows = []
    jdk_configs = {
        'javac8': (Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
                   ['-source', '8', '-target', '8']),
        'javac23': (Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'),
                    ['--release', '8']),
    }

    reflect_source = (EVIDENCE / 'ReflectDriver.java').read_text()
    reflect_source = reflect_source.replace(
        'if (parameter.isArray())',
        'if (parameter==boolean.class) return Boolean.TRUE;\n'
        '    if (Map.class.isAssignableFrom(parameter)) return new HashMap<String,String>(Collections.singletonMap("raw","value"));\n'
        '    if (parameter.isArray())')
    reflect_source = reflect_source.replace(
        'else m.invoke(o,argument(m.getParameterTypes()[0],method,c));',
        'else { Class<?>[] ps=m.getParameterTypes(); Object[] xs=new Object[ps.length]; '
        'for(int k=0;k<ps.length;k++) xs[k]=argument(ps[k],method,c); m.invoke(o,xs); }')
    reflect_template = OUT / 'ReflectDriver.java'
    reflect_template.write_text(reflect_source)
    probe_template = OUT / 'ProbeDriver.java'
    probe_template.write_text(PROBE_DRIVER)
    libs = sorted((JADX.parent.parent / 'lib').glob('*.jar'))
    if len(libs) != 57:
        raise SystemExit(f'expected 57 reference JADX libraries, got {len(libs)}')

    jdk_manifest = {}
    for leg, (home, release) in jdk_configs.items():
        jdk_manifest[leg] = {
            'home': str(home), 'release_flags': release,
            'java_sha256': sha(home / 'bin/java'), 'javac_sha256': sha(home / 'bin/javac'),
            'jar_sha256': sha(home / 'bin/jar'), 'javap_sha256': sha(home / 'bin/javap'),
        }
        driver_dir = OUT / 'drivers' / leg
        driver_dir.mkdir(parents=True)
        driver_cp = OUT / 'empty-classpath' / leg
        driver_sp = OUT / 'empty-sourcepath' / leg
        driver_cp.mkdir(); driver_sp.mkdir()
        driver_out = OUT / 'tool-logs' / leg
        driver_out.mkdir(parents=True)
        run(commands, 'compile-reference-drivers', [home / 'bin/javac', *release, '-g:none',
            '-classpath', driver_cp, '-sourcepath', driver_sp, '-d', driver_dir,
            reflect_template, probe_template], OUT, driver_out, 'drivers-javac')
        # Drivers must be available to every runtime and are independent of each tested class.
        if commands[-1]['exit'] != 0:
            raise SystemExit(f'reference drivers failed to compile for {leg}')
        for tool, args in [('java', ['-version']), ('javac', ['-version']),
                           ('jar', ['-help'] if leg == 'javac8' else ['--version'])]:
            run(commands, 'tool-version-' + leg + '-' + tool,
                [home / 'bin' / tool, *args], OUT, driver_out, tool + '-version')

    for old in targets:
        family, leg = old['family'], old['leg']
        home, release = jdk_configs[leg]
        source_path = Path(old['source']['path'])
        if sha(source_path) != old['source']['sha256']:
            raise SystemExit(f'source SHA changed: {source_path}')
        case = results / leg / family
        case.mkdir(parents=True)
        cp = OUT / 'empty-classpath' / leg
        sp = OUT / 'empty-sourcepath' / leg
        for d in ('original', 'jadx', 'baseline', 'candidate'):
            (case / 'compile-sources' / d).mkdir(parents=True)
        source_copy = case / 'input' / source_path.name
        source_copy.parent.mkdir()
        shutil.copyfile(source_path, source_copy)
        original_classes = inputs / leg / family / 'classes'
        original_classes.mkdir(parents=True)
        original_jar = inputs / leg / family / f'{family}.jar'
        original_jar.parent.mkdir(parents=True, exist_ok=True)
        original_log = case / 'commands'; original_log.mkdir()
        historical_javac = [command for command in metadata['actual_commands']
            if Path(command['argv'][0]).name == 'javac'
            and str(source_path) in command['argv']
            and str(Path(command['argv'][command['argv'].index('-d') + 1])).endswith(f'/{family}/original')
            and (leg == 'javac8' and 'corretto-1.8.0_432' in command['argv'][0]
                 or leg == 'javac23' and 'openjdk-23.0.1' in command['argv'][0])]
        historical_jar = [command for command in metadata['actual_commands']
            if Path(command['argv'][0]).name == 'jar'
            and old['temporary_path'] in command['argv']]
        if len(historical_javac) != 1 or len(historical_jar) != 1:
            raise SystemExit(f'CLI8 original javac/jar provenance is not unique: {leg}/{family}')
        original_compile = run(commands, 'original-source-javac',
            [home / 'bin/javac', *release, '-g:none', '-classpath', cp, '-sourcepath', sp,
             '-d', original_classes, source_copy], case, original_log, 'original-javac')
        if original_compile.returncode != 0:
            raise SystemExit(f'original source did not compile: {leg}/{family}')
        rebuilt_class_hashes = hash_classes(original_classes)
        jar_run = run(commands, 'permanent-original-jar',
            [home / 'bin/jar', 'cf', original_jar, '-C', original_classes, '.'],
            case, original_log, 'original-jar')
        if jar_run.returncode != 0:
            raise SystemExit(f'original JAR failed: {leg}/{family}')
        old_class_hashes = old['class_sha256']
        class_hash_match = rebuilt_class_hashes == old_class_hashes
        copied_source_hash = sha(source_copy)

        original_runtime = case / 'original-runtime'; original_runtime.mkdir()
        is_probe = source_path.parent.name == family and 'root-probes' in source_path.as_posix()
        driver = 'ProbeDriver' if is_probe else 'ReflectDriver'
        runtime_args = [family] if is_probe else reflect_args(family)
        input_driver_sources = [probe_template] if is_probe else [reflect_template]
        original_run = run(commands, 'original-input-runtime',
            [home / 'bin/java', '-Xverify:all', '-cp',
             str(original_classes) + ':' + str(OUT / 'drivers' / leg), driver, *runtime_args],
            case, original_runtime, 'run')

        javap_log = case / 'javap'; javap_log.mkdir()
        javap = run(commands, 'original-input-javap',
            [home / 'bin/javap', '-p', '-s', '-v', '-classpath', original_jar, family],
            case, javap_log, 'javap')

        jadx_dir = case / 'jadx'; jadx_dir.mkdir()
        jadx_out = jadx_dir / 'full-output'; jadx_out.mkdir()
        jadx_run = run(commands, 'reference-jadx',
            [JADX, '--no-res', '-d', jadx_out, original_jar], case, jadx_dir, 'jadx')
        jadx_sources = sorted(jadx_out.rglob('*.java'))

        reports = {}
        source_variants = {'original': [source_copy], 'jadx': jadx_sources}
        cli_runs = {}
        for version, cli in [('baseline', BASELINE), ('candidate', CANDIDATE)]:
            cli_dir = case / version; cli_dir.mkdir()
            result = run(commands, version + '-class-source',
                [cli, 'class-source', '--input', original_jar, '--class', family,
                 '--policy', 'plain-jar', '--format', 'json', '--evidence', 'all', '--release', '8'],
                case, cli_dir, 'class-source')
            cli_runs[version] = result
            if result.returncode not in (0, 4):
                raise SystemExit(f'{version} class-source failed unexpectedly: {leg}/{family}')
            report = json.loads(result.stdout)
            reports[version] = report
            (cli_dir / 'report.json').write_bytes(result.stdout)
            java_file = cli_dir / f'{family}.java'
            java_file.write_text(report['text'])
            source_variants[version] = [java_file]

        class_identity = class_name([source_copy], family)
        flavors = {}
        for flavor, flavor_sources in source_variants.items():
            flavor_dir = case / 'compile-sources' / flavor
            copied = []
            for i, source in enumerate(flavor_sources):
                target = flavor_dir / source.name
                if target.exists():
                    target = flavor_dir / f'{i}-{source.name}'
                shutil.copyfile(source, target)
                copied.append(target)
            classes = case / 'classes' / flavor
            classes.mkdir(parents=True)
            compile_run = run(commands, flavor + '-isolated-javac',
                [home / 'bin/javac', *release, '-g:none', '-classpath', cp,
                 '-sourcepath', sp, '-d', classes, *copied], case, flavor_dir, flavor + '-javac')
            flavor_data = {
                'sources': [{'path': str(p.relative_to(case)), 'sha256': sha(p)} for p in copied],
                'compile_exit': compile_run.returncode,
                'compiled_classes_sha256': hash_classes(classes),
                'classpath': str(cp), 'sourcepath': str(sp),
            }
            if compile_run.returncode == 0:
                runtime_name = 'original-recompiled-runtime' if flavor == 'original' else flavor + '-runtime'
                runtime_dir = case / runtime_name; runtime_dir.mkdir()
                execution = run(commands, flavor + '-Xverify-runtime',
                    [home / 'bin/java', '-Xverify:all', '-cp',
                     str(classes) + ':' + str(OUT / 'drivers' / leg), driver,
                     *([class_identity] if is_probe else runtime_args)],
                    case, runtime_dir, 'run')
                output = execution.stdout.decode(errors='replace')
                original_output = original_run.stdout.decode(errors='replace')
                flavor_data.update({
                    'runtime_exit': execution.returncode,
                    'runtime_stdout_sha256': sha(runtime_dir / 'run.stdout'),
                    'runtime_stderr_sha256': sha(runtime_dir / 'run.stderr'),
                    'behavior_match': semantic(output) == semantic(original_output),
                    'api_output_match': execution.stdout == original_run.stdout,
                })
                (runtime_dir / 'behavior.diff').write_text(''.join(difflib.unified_diff(
                    semantic(original_output), semantic(output), fromfile='original', tofile=flavor)))
                (runtime_dir / 'api.diff').write_text(''.join(difflib.unified_diff(
                    original_output.splitlines(True), output.splitlines(True),
                    fromfile='original-api', tofile=flavor + '-api')))
            flavors[flavor] = flavor_data

        rows.append({
            'leg': leg, 'family': family, 'source_path': str(source_path),
            'source_sha256': old['source']['sha256'], 'source_copy_sha256': copied_source_hash,
            'historical_temp_jar_path': old['temporary_path'],
            'historical_temp_jar_sha256': old['jar_sha256'],
            'historical_temp_jar_present_after_cli8': Path(old['temporary_path']).is_file(),
            'historical_original_javac_command': historical_javac[0],
            'historical_original_jar_command': historical_jar[0],
            'historical_class_sha256': old_class_hashes,
            'permanent_rebuilt_jar': str(original_jar), 'permanent_rebuilt_jar_sha256': sha(original_jar),
            'rebuilt_class_sha256': rebuilt_class_hashes, 'class_hash_matches_historical': class_hash_match,
            'original_compile_exit': original_compile.returncode, 'original_jar_exit': jar_run.returncode,
            'original_runtime_exit': original_run.returncode, 'original_runtime_stdout': str(original_runtime / 'run.stdout'),
            'original_runtime_stderr': str(original_runtime / 'run.stderr'), 'javap_exit': javap.returncode,
            'jadx_exit': jadx_run.returncode,
            'jadx_sources': [{'path': str(p.relative_to(case)), 'sha256': sha(p)} for p in jadx_sources],
            'cli': {version: {'exit': cli_runs[version].returncode,
                              'stdout_sha256': sha(case / version / 'class-source.stdout'),
                              'stderr_sha256': sha(case / version / 'class-source.stderr'),
                              'report_sha256': sha(case / version / 'report.json'),
                              'source_sha256': sha(case / version / f'{family}.java')}
                    for version in ('baseline', 'candidate')},
            'flavors': flavors,
        })

    snapshot = OUT / 'replay-executed.py'
    manifest = {
        'scope': 'Only the 18 CLI8 source-rebuilt original inputs (9 families x javac8/javac23); original temp JAR bytes were not preserved by CLI8 and are not claimed identical.',
        'candidate_cli': str(CANDIDATE), 'candidate_cli_sha256': sha(CANDIDATE),
        'baseline_cli': str(BASELINE), 'baseline_cli_sha256': sha(BASELINE),
        'historical_cli8_run_metadata': str(META), 'historical_cli8_run_metadata_sha256': sha(META),
        'historical_cli8_source_rebuilt_inputs': 18,
        'historical_jar_sha256_limitation': 'Historical temporary JAR paths no longer exist. Rebuilt permanent JAR hashes are recorded separately; only matching .class SHA proves class bytes equal, not JAR bytes or ZIP timestamps.',
        'jadx': str(JADX), 'jadx_sha256': sha(JADX),
        'jadx_libs': [{'path': str(p), 'sha256': sha(p)} for p in libs],
        'jdk_tools': jdk_manifest,
        'drivers': {
            'reflect_driver_source': str(reflect_template), 'reflect_driver_sha256': sha(reflect_template),
            'probe_driver_source': str(probe_template), 'probe_driver_sha256': sha(probe_template),
        },
        'isolation': 'Every source javac invocation uses this leg’s explicit empty -classpath and -sourcepath; every runtime uses -Xverify:all and only fresh flavor classes plus separately compiled external driver classes. Original input JAR is not on runtime classpath.',
        'probe_driver_families': sorted(row['family'] for row in rows
                                        if 'root-probes' in row['source_path']),
        'commands': commands, 'cases': rows,
    }
    manifest['files'] = [
        {'path': str(path.relative_to(OUT)), 'bytes': path.stat().st_size, 'sha256': sha(path)}
        for path in sorted(OUT.rglob('*')) if path.is_file() and path.name != 'manifest.json'
    ]
    (OUT / 'manifest.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({
        'out': str(OUT), 'families_legs': len(rows),
        'class_hash_matches_historical': sum(row['class_hash_matches_historical'] for row in rows),
        'jadx_compiled': sum(row['flavors']['jadx']['compile_exit'] == 0 for row in rows),
        'baseline_compiled': sum(row['flavors']['baseline']['compile_exit'] == 0 for row in rows),
        'candidate_compiled': sum(row['flavors']['candidate']['compile_exit'] == 0 for row in rows),
        'command_count': len(commands),
    }, indent=2))


if __name__ == '__main__':
    main()
