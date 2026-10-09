#!/usr/bin/env python3
"""Reproduce the EM18 v3 JADX comparison without touching the fixture."""
from __future__ import annotations
import hashlib, json, os, re, shutil, subprocess, sys, zipfile
from pathlib import Path

REPO = Path('/Users/lordcasser/workspace/projects/jarde')
FIXTURE = REPO / 'tests/fixtures/p3-heterogeneous-array-initializers-v3'
OUT = Path('/private/tmp/jarde-em18-jadx-v3-reference')
JADX = Path('/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx')
JADX_ROOT = JADX.parent.parent
JADX_REPO = Path('/Users/lordcasser/workspace/testzone/jadx')
SCHEMA = 'jarde-em18-jadx-v3-reference-v1'


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def file_sha(path: Path) -> str:
    return sha(path.read_bytes())

def run(argv, cwd: Path, stdout_path: Path, stderr_path: Path):
    cp = subprocess.run([str(x) for x in argv], cwd=str(cwd), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stdout_path.write_bytes(cp.stdout)
    stderr_path.write_bytes(cp.stderr)
    return {
        'argv': [str(x) for x in argv], 'cwd': str(cwd), 'exit': cp.returncode,
        'stdout': str(stdout_path.relative_to(OUT)), 'stdout_sha256': sha(cp.stdout),
        'stderr': str(stderr_path.relative_to(OUT)), 'stderr_sha256': sha(cp.stderr),
    }

def java_home_for(leg: str, original_manifest: dict) -> Path:
    return Path(original_manifest['families']['factory']['legs'][leg]['home'])

def main_fqcn(main_source: str) -> str:
    package = re.search(r'^\s*package\s+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)\s*;', main_source, re.M)
    return (package.group(1) + '.' if package else '') + 'Main'

def method_body(source: str, method: str) -> str | None:
    match = re.search(r'\b' + re.escape(method) + r'\s*\([^)]*\)\s*(?:throws\s+[^\{]+)?\{', source)
    if not match:
        return None
    depth = 1
    i = match.end()
    while i < len(source) and depth:
        if source[i] == '{': depth += 1
        elif source[i] == '}': depth -= 1
        i += 1
    return source[match.end():i-1] if depth == 0 else None

def initializer_methods(main_source: str, names: list[str]) -> dict[str, bool]:
    result = {}
    for name in names:
        body = method_body(main_source, name)
        result[name] = bool(body and re.search(r'new\s+[A-Za-z_$][\w.$<>?, ]*(?:\[\])+\s*\{', body))
    return result

def main_run():
    manifest_path = FIXTURE / 'build-manifest-v3-final.json'
    original = json.loads(manifest_path.read_text())
    existing = [p for p in OUT.iterdir() if p.name != 'run_reference.py'] if OUT.exists() else []
    if existing and '--reset' not in sys.argv[1:]:
        raise SystemExit(f'evidence already exists; pass --reset to replace only this evidence directory: {existing[0]}')
    if '--reset' in sys.argv[1:]:
        for p in existing:
            shutil.rmtree(p) if p.is_dir() else p.unlink()
    OUT.mkdir(parents=True, exist_ok=True)
    input_dir = OUT / 'inputs'
    input_dir.mkdir()
    empty = OUT / 'empty-classpath-sourcepath'
    empty.mkdir()

    input_jars = {}
    for family in ('factory', 'direct'):
        for leg in ('javac8', 'javac23'):
            leg_key = leg.replace('javac', 'javac')
            classes_dir = FIXTURE / family / leg / 'classes'
            jar_path = input_dir / f'{family}-{leg}.jar'
            entries = []
            with zipfile.ZipFile(jar_path, 'w', compression=zipfile.ZIP_STORED) as zf:
                for class_path in sorted(classes_dir.glob('*.class')):
                    payload = class_path.read_bytes()
                    info = zipfile.ZipInfo(class_path.name, date_time=(2026, 10, 9, 0, 0, 0))
                    info.compress_type = zipfile.ZIP_STORED
                    info.external_attr = 0o100644 << 16
                    zf.writestr(info, payload)
                    entries.append({'name': class_path.name, 'bytes': len(payload), 'sha256': sha(payload)})
            expected = {x['path']: x['sha256'] for x in original['families'][family]['legs'][leg]['class_files']}
            actual = {x['name']: x['sha256'] for x in entries}
            if actual != expected:
                raise SystemExit(f'fixture class hash mismatch for {family}/{leg}: expected {expected}, actual {actual}')
            input_jars[(family, leg)] = {'path': str(jar_path.relative_to(OUT)), 'sha256': file_sha(jar_path), 'bytes': jar_path.stat().st_size, 'entries': entries}

    launcher_hash = file_sha(JADX)
    version = run([JADX, '--version'], JADX.parent, OUT/'identity/jadx-version.stdout', OUT/'identity/jadx-version.stderr')
    jdk_identity = {}
    for leg in ('javac8', 'javac23'):
        home = java_home_for(leg, original)
        jdk_identity[leg] = {}
        for tool in ('java', 'javac'):
            path = home / 'bin' / tool
            result = run([path, '-version'], FIXTURE, OUT/f'identity/{leg}-{tool}-version.stdout', OUT/f'identity/{leg}-{tool}-version.stderr')
            result.update({'path': str(path), 'sha256': file_sha(path)})
            jdk_identity[leg][tool] = result

    profiles = {'default': [], 'rename-flags-none': ['--rename-flags', 'none']}
    method_names = {
        'factory': ['boxedFactory','sequenceFactory','collectionFactory','throwableFactory','numberGridFactory','collectionGridFactory','ownTwoHopFactory','ownInterfaceFactory','ownGridFactory'],
        'direct': ['boxedDirect','sequenceDirect','collectionDirect','throwableDirect','numberGridDirect','collectionGridDirect','ownTwoHopDirect','ownInterfaceDirect','ownGridDirect'],
    }
    runs = []
    for family in ('factory', 'direct'):
        for leg in ('javac8', 'javac23'):
            for profile, profile_flags in profiles.items():
                key = f'{family}/{leg}/{profile}'
                work = OUT / 'runs' / family / leg / profile
                jadx_out = work / 'jadx'
                source_root = jadx_out / 'sources'
                jadx_out.mkdir(parents=True)
                jar_path = OUT / input_jars[(family, leg)]['path']
                jadx_argv = [JADX, *profile_flags, '-d', jadx_out, jar_path]
                jadx_command = run(jadx_argv, FIXTURE, work/'logs/jadx.stdout', work/'logs/jadx.stderr')
                source_files = sorted(source_root.rglob('*.java')) if source_root.exists() else []
                source_records = []
                for source in source_files:
                    content = source.read_bytes()
                    source_records.append({'path': str(source.relative_to(OUT)), 'bytes': len(content), 'sha256': sha(content)})
                main_candidates = [p for p in source_files if re.search(r'\bclass\s+Main\b', p.read_text(errors='replace'))]
                entry = None
                main_text = None
                if len(main_candidates) == 1:
                    main_text = main_candidates[0].read_text(errors='replace')
                    entry = main_fqcn(main_text)
                classes_out = work / 'compiled-classes'
                classes_out.mkdir()
                home = java_home_for(leg, original)
                javac = home / 'bin' / 'javac'
                compile_flags = ['-source','8','-target','8','-g:none'] if leg == 'javac8' else ['--release','8','-g:none']
                compile_argv = [javac, *compile_flags, '-classpath', empty, '-sourcepath', empty, '-d', classes_out, *source_files]
                if source_files:
                    compile_command = run(compile_argv, FIXTURE, work/'logs/javac.stdout', work/'logs/javac.stderr')
                else:
                    compile_command = {'argv':[str(x) for x in compile_argv], 'cwd':str(FIXTURE), 'exit':None, 'skipped':'no generated Java sources', 'stdout':None, 'stderr':None}
                runtime_command = None
                if compile_command['exit'] == 0 and entry:
                    java = home / 'bin' / 'java'
                    runtime_command = run([java, '-Xverify:all', '-cp', classes_out, entry], FIXTURE, work/'logs/java.stdout', work/'logs/java.stderr')
                original_leg = original['families'][family]['legs'][leg]
                orig_out = (FIXTURE / original_leg['original_stdout']).read_bytes()
                orig_err = (FIXTURE / original_leg['original_stderr']).read_bytes()
                runtime_out = (OUT / runtime_command['stdout']).read_bytes() if runtime_command and runtime_command['stdout'] else None
                runtime_err = (OUT / runtime_command['stderr']).read_bytes() if runtime_command and runtime_command['stderr'] else None
                # Scan only generated Main.java, with a small brace-aware method extraction.
                accepted_map = initializer_methods(main_text or '', method_names[family])
                accepted = sum(accepted_map.values())
                runs.append({
                    'key': key, 'family': family, 'jdk_leg': leg, 'profile': profile,
                    'input_jar': input_jars[(family,leg)], 'jadx': jadx_command,
                    'generated_source_root': str(source_root.relative_to(OUT)),
                    'generated_sources': source_records, 'main_source': str(main_candidates[0].relative_to(OUT)) if len(main_candidates)==1 else None,
                    'entry_class': entry, 'compile': compile_command, 'runtime': runtime_command,
                    'original': {
                        'source_hashes': original['families'][family]['sources'],
                        'class_files': original_leg['class_files'],
                        'stdout': original_leg['original_stdout'], 'stdout_sha256': original_leg['original_stdout_sha256'],
                        'stderr': original_leg['original_stderr'], 'stderr_sha256': original_leg['original_stderr_sha256'],
                        'exit': original_leg['original_runtime_exit'],
                    },
                    'comparison': {
                        'compile_succeeded': compile_command['exit'] == 0,
                        'runtime_exit_matches': runtime_command is not None and runtime_command['exit'] == original_leg['original_runtime_exit'],
                        'stdout_exact': runtime_out == orig_out if runtime_out is not None else False,
                        'stderr_exact': runtime_err == orig_err if runtime_err is not None else False,
                    },
                    'initializer_methods_expected': method_names[family],
                    'initializer_methods_detected': accepted_map,
                    'initializer_methods_accepted_count': accepted,
                    'initializer_methods_total': len(method_names[family]),
                })

    # Bundle fixture provenance by source/class/raw stdout SHA without copying or modifying any input.
    provenance = {}
    for family in ('factory','direct'):
        for leg in ('javac8','javac23'):
            data = original['families'][family]['legs'][leg]
            provenance[f'{family}/{leg}'] = {
                'sources': original['families'][family]['sources'],
                'classes': data['class_files'],
                'original_exit': data['original_runtime_exit'],
                'original_stdout': {'path': data['original_stdout'], 'sha256': data['original_stdout_sha256']},
                'original_stderr': {'path': data['original_stderr'], 'sha256': data['original_stderr_sha256']},
                'input_jar': input_jars[(family,leg)],
            }
    jadx_lib_hashes = {}
    for p in sorted((JADX_ROOT/'lib').glob('*.jar')):
        jadx_lib_hashes[p.name] = file_sha(p)
    head = (JADX_REPO/'.git/HEAD').read_text().strip()
    ref_name = head.removeprefix('ref: ').strip() if head.startswith('ref: ') else None
    ref_path = JADX_REPO/'.git'/ref_name if ref_name else None
    git_sha = ref_path.read_text().strip() if ref_path and ref_path.exists() else None
    summary = {
        'runs': len(runs),
        'compile_successes': sum(bool(r['comparison']['compile_succeeded']) for r in runs),
        'exact_runtime_matches': sum(all(r['comparison'][k] for k in ('runtime_exit_matches','stdout_exact','stderr_exact')) for r in runs),
        'runtime_attempts': sum(r['runtime'] is not None for r in runs),
        'initializer_methods_accepted_total': sum(r['initializer_methods_accepted_count'] for r in runs),
        'initializer_methods_total': sum(r['initializer_methods_total'] for r in runs),
        'exact_by_family_profile': {},
        'accepted_by_family_profile': {},
    }
    for family in ('factory','direct'):
        for profile in profiles:
            group = [r for r in runs if r['family']==family and r['profile']==profile]
            summary['exact_by_family_profile'][f'{family}/{profile}'] = sum(all(r['comparison'][k] for k in ('runtime_exit_matches','stdout_exact','stderr_exact')) for r in group)
            summary['accepted_by_family_profile'][f'{family}/{profile}'] = {'accepted':sum(r['initializer_methods_accepted_count'] for r in group),'total':sum(r['initializer_methods_total'] for r in group)}
    result = {
        'schema': SCHEMA,
        'fixture': str(FIXTURE), 'fixture_manifest': str(manifest_path), 'fixture_manifest_sha256': file_sha(manifest_path),
        'runner': str(Path(__file__)), 'runner_sha256': file_sha(Path(__file__)),
        'created_local': subprocess.run(['date','-u','+%Y-%m-%dT%H:%M:%SZ'],stdout=subprocess.PIPE,check=True).stdout.decode().strip(),
        'tool_identity': {
            'jadx_launcher': str(JADX), 'launcher_sha256': launcher_hash, 'version_command': version,
            'reported_version': (OUT/'identity/jadx-version.stdout').read_text().strip(),
            'jadx_source_repo': str(JADX_REPO), 'git_head_ref': head, 'git_sha_from_ref_file': git_sha,
            'source_worktree_cleanliness': 'not queried (Git commands prohibited by task)',
            'lib_jar_sha256': jadx_lib_hashes,
        },
        'jdk_identity': jdk_identity,
        'original_provenance': provenance,
        'empty_classpath_sourcepath': str(empty),
        'profiles': {k: v for k,v in profiles.items()},
        'summary': summary,
        'runs': runs,
    }
    manifest_out = OUT/'manifest.json'
    manifest_out.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps(summary, indent=2))
    print(f'manifest={manifest_out}')

if __name__ == '__main__':
    main_run()
