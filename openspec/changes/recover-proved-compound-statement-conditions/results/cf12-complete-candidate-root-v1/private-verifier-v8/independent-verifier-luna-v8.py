#!/usr/bin/env python3
"""Independent, read-only verifier for a root-executed CF12 replay record."""
import argparse
import ast
import collections
import hashlib
import json
import os
import re
import sys
import zipfile
from pathlib import Path

PLAN = Path('/private/tmp/jarde-cf12-ten-fixture-replay-plan-luna-v1.json')
JAVAP_EVIDENCE = Path('/private/tmp/jarde-switch-continuation-physical-javap-root-v1/execution.json')
SIDES = ('original', 'jadx', 'jarde-default', 'jarde-all')
JDKS = ('jdk8_tools', 'jdk23_tools')
TARGETS = {
    'TestSwitch2': ('test', '(I)V'),
    'TestSwitchWithFallThroughCase2': ('test', '(IZZ)Ljava/lang/String;'),
}

class VerifyError(Exception):
    pass

def need(ok, message):
    if not ok:
        raise VerifyError(message)

def canonical(x):
    return json.dumps(x, ensure_ascii=False, sort_keys=True, separators=(',', ':'))

def sha_bytes(b):
    return hashlib.sha256(b).hexdigest()

def digest_file(path):
    p = Path(path)
    b = p.read_bytes()
    return {'path': str(p), 'bytes': len(b), 'sha256': sha_bytes(b)}, b

def verify_pin(pin):
    actual, _ = digest_file(pin['path'])
    need(actual['bytes'] == pin['bytes'] and actual['sha256'] == pin['sha256'],
         f"pin mismatch: {pin.get('path')}")

def raw_for(command):
    streams = command.get('streams')
    need(isinstance(streams, dict), f"missing streams: {command.get('label')}")
    result = {}
    for stream in ('stdout', 'stderr'):
        pin = streams.get(stream)
        need(isinstance(pin, dict), f"missing {stream} pin: {command.get('label')}")
        verify_pin(pin)
        result[stream] = Path(pin['path']).read_bytes()
    return result

def legacy_raw_for(command):
    streams=command.get('streams')
    if isinstance(streams,dict):
        return raw_for(command)
    need(isinstance(streams,list) and len(streams)==2,
         f"legacy command streams must be exactly stdout/stderr pins: {command.get('label')}")
    result={}
    for pin in streams:
        path=pin.get('path','')
        name='stdout' if path.endswith('.stdout.raw') else 'stderr' if path.endswith('.stderr.raw') else None
        need(name is not None and name not in result, f"ambiguous legacy stream suffix: {path}")
        actual,data=digest_file(path)
        need(actual['bytes']==pin['bytes'] and actual['sha256']==pin['sha256'],f'legacy raw pin mismatch: {path}')
        result[name]=data
    need(set(result)=={'stdout','stderr'},'legacy command missing stdout/stderr raw')
    return result


def command_index(execution):
    rows = execution.get('commands')
    need(isinstance(rows, list), 'commands missing')
    by_label = {}
    for cmd in rows:
        need(isinstance(cmd.get('argv'), list) and cmd['argv'], 'command argv missing')
        need(isinstance(cmd.get('exit_code'), int), f"exit code missing: {cmd.get('label')}")
        raw_for(cmd)
        by_label.setdefault(cmd.get('label'), []).append(cmd)
    return by_label

def command_matches_embedded(cmd, index):
    label = cmd.get('label')
    candidates = index.get(label, [])
    need(any(canonical(c) == canonical(cmd) for c in candidates), f'embedded command differs from raw command list: {label}')

def method_report(method):
    out = method.get('outcome')
    need(isinstance(out, dict), 'method outcome missing')
    report = out.get('report')
    need(isinstance(report, dict), 'method report missing')
    need(isinstance(report.get('text'), str), 'method report text missing')
    sm = report.get('source_map')
    need(isinstance(sm, dict) and isinstance(sm.get('segments'), list), 'method source_map/segments missing')
    return report, sm

def verify_source_map(method, fixture, expected_identity, b3, expected_length):
    report, sm = method_report(method)
    encoded = report['text'].encode('utf-8')
    need(method.get('item', {}).get('identity') == expected_identity,
         f'{fixture}: item identity differs from expected physical identity')
    owner = expected_identity.get('owner', {})
    need(owner.get('class_bytes', {}).get('digest') == b3,
         f'{fixture}: declaration owner digest differs from pinned physical class BLAKE3')
    need(owner.get('class_bytes', {}).get('length') == expected_length,
         f'{fixture}: declaration owner byte length differs from pinned class')
    location = owner.get('location', {})
    if location.get('kind') == 'standalone_root':
        need(location.get('snapshot') == b3, f'{fixture}: standalone reader snapshot differs from pinned class digest')
    binding = report.get('artifact', {}).get('binding')
    need(isinstance(binding, dict) and binding,
         f'{fixture}: artifact.binding must be present and nonempty')
    need(binding.get('method') == expected_identity,
         f'{fixture}: artifact.binding.method differs from actual declaration identity')
    need(binding.get('member_ordinal') == method['item'].get('index'),
         f'{fixture}: artifact.binding.member_ordinal differs from actual declaration index')
    env_content = binding.get('environment', {}).get('content', [])
    need(isinstance(env_content, list) and b3 in env_content,
         f'{fixture}: binding reader content does not identify pinned physical class')
    need(isinstance(method['item'].get('index'), int), f'{fixture}: declaration index missing')
    for seg in sm['segments']:
        start, end = seg.get('start'), seg.get('end')
        need(isinstance(start, int) and isinstance(end, int) and 0 <= start < end <= len(encoded),
             f'{fixture}: empty/out-of-range UTF-8 span {start}:{end}')
        try:
            encoded[:start].decode('utf-8')
            encoded[:end].decode('utf-8')
        except UnicodeDecodeError:
            raise VerifyError(f'{fixture}: source span splits a UTF-8 code point at {start}:{end}')
        origin = seg.get('origin')
        need(isinstance(origin, dict), f'missing source map origin: {fixture}')
        facts = ([origin['primary']] if origin.get('primary') else []) + origin.get('derived', [])
        need(facts, f'{fixture}: source map segment has no provenance facts')
        for fact in facts:
            need(isinstance(fact, dict), f'bad source fact: {fixture}')
            physical = fact.get('method')
            need(isinstance(physical, dict) and physical == expected_identity,
                 f'{fixture}: source-map fact is attributed to a different physical method')
            need(isinstance(fact.get('bci'), int) and fact['bci'] >= 0,
                 f'BCI missing/invalid: {fixture}')
    return report, sm

def descriptor_key(method):
    item = method.get('item')
    need(isinstance(item, dict), 'method item missing')
    identity = item.get('identity')
    need(isinstance(identity, dict), 'method identity missing')
    name = item.get('name')
    desc = item.get('descriptor')
    need(isinstance(name, dict) and isinstance(desc, dict), 'method name/descriptor object missing')
    return identity, name, desc

def validate_api_argv(cmd, api_row, side, cli_path):
    inp = str(Path(api_row['input']['path']).resolve())
    internal = api_row.get('class')
    need(isinstance(internal, str) and internal, 'class-source internal class name missing')
    expected = [str(Path(cli_path).resolve()), 'class-source', '--input', inp, '--class', internal,
                '--policy', 'single-class', '--release', '8', '--format', 'json']
    if side == 'jarde-all':
        expected += ['--evidence', 'all']
    need(cmd.get('argv') == expected, f"class-source argv differs from required {side} policy/input/release: {cmd.get('label')}")


def api_doc(api_row, idx, side=None, cli_path=None):
    cmd = api_row.get('command')
    need(isinstance(cmd, dict), 'class-source command missing')
    command_matches_embedded(cmd, idx)
    if side is not None:
        validate_api_argv(cmd, api_row, side, cli_path)
    raws = raw_for(cmd)
    need(cmd['exit_code'] == 0, f"class-source failed: {cmd.get('label')}")
    try:
        doc = json.loads(raws['stdout'].decode('utf-8'))
    except Exception as exc:
        raise VerifyError(f"class-source stdout is not JSON: {cmd.get('label')}: {exc}")
    need(doc.get('outcome') == 'performed', f"class-source outcome not performed: {cmd.get('label')}")
    need(isinstance(doc.get('text'), str) and doc['text'], f"full class source missing: {cmd.get('label')}")
    need(isinstance(doc.get('methods'), list), f"class source methods missing: {cmd.get('label')}")
    return doc

def check_tool_argv(cmd, tool_pin, role):
    need(Path(cmd['argv'][0]).resolve() == Path(tool_pin['path']).resolve(),
         f"{cmd.get('label')}: argv executable is not pinned {role}")
    verify_pin(tool_pin)


def class_set(side):
    return side.get('compiled_classes') if 'compiled_classes' in side else side.get('classes')

def javap_instructions(text):
    methods = {}
    name = None
    desc = None
    inst = set()
    for line in text.splitlines():
        m = re.match(r'^  .+\(.*\);$', line)
        if m:
            if name is not None and desc is not None:
                methods[(name, desc)] = inst
            head = line.strip().split('(')[0].split()[-1]
            name = '<init>' if head.endswith('.<init>') else head
            desc = None
            inst = set()
        elif name is not None and line.strip().startswith('descriptor:'):
            desc = line.strip().split(':', 1)[1].strip()
        else:
            m = re.match(r'^\s+(\d+):\s+[a-z][a-z0-9_]*\b', line)
            if m and name is not None:
                inst.add(int(m.group(1)))
    if name is not None and desc is not None:
        methods[(name, desc)] = inst
    return methods

def find_javap(execution, fixture, input_path):
    # Root’s frozen JDK 23 javap evidence is separately pinned and contains the
    # actual argv/raw stream pins; do not infer BCIs from collector metadata.
    manifest = json.loads(JAVAP_EVIDENCE.read_text())
    tool = manifest.get('tool', {})
    tool_pin, _ = digest_file(tool.get('path', ''))
    need(tool_pin['sha256'] == tool.get('sha256'), 'frozen javap tool pin mismatch')
    rows = [r for r in manifest.get('commands', []) if r.get('fixture') == fixture]
    need(len(rows) == 1, f'expected one frozen javap record for {fixture}')
    row = rows[0]
    need(Path(row['input']['path']).resolve() == Path(input_path).resolve(), f'{fixture}: javap evidence input mismatch')
    input_pin, _ = digest_file(input_path)
    need(input_pin['bytes'] == row['input']['bytes'] and input_pin['sha256'] == row['input']['sha256'], f'{fixture}: javap input pin mismatch')
    argv = row.get('argv', [])
    need(argv and Path(argv[0]).resolve() == Path(tool['path']).resolve(), f'{fixture}: javap argv tool mismatch')
    need(all(x in argv for x in ('-v', '-p', '-c')), f'{fixture}: javap argv lacks -v -p -c')
    need(any(Path(x).resolve() == Path(input_path).resolve() for x in argv), f'{fixture}: javap argv lacks pinned class')
    need(row.get('exit_code') == 0, f'{fixture}: frozen javap failed')
    raw = {}
    for stream in ('stdout', 'stderr'):
        entry = row[stream]
        actual, data = digest_file(entry['path'])
        need(actual['bytes'] == entry['bytes'] and actual['sha256'] == entry['sha256'], f'{fixture}: javap {stream} pin mismatch')
        raw[stream] = data
    return javap_instructions(raw['stdout'].decode('utf-8', errors='strict'))

def verify_changed_method(doc, fixture, name, desc, b3, class_length, javap):
    found = []
    for method in doc['methods']:
        identity, item_name, item_desc = descriptor_key(method)
        escaped_name = item_name.get('escaped')
        escaped_desc = item_desc.get('escaped')
        if (escaped_name, escaped_desc) == (name, desc):
            found.append((method, identity, item_name, item_desc))
    need(len(found) == 1, f'{fixture}: expected exactly one target method {name}{desc}, found {len(found)}')
    method, identity, item_name, item_desc = found[0]
    report, sm = verify_source_map(method, fixture, identity, b3, class_length)
    need(item_name.get('raw') == identity.get('name') and item_desc.get('raw') == identity.get('descriptor'),
         f'{fixture}: item name/descriptor raw bytes differ from identity')
    key = (name, desc)
    need(key in javap, f'{fixture}: actual javap lacks method {key}')
    actual_bcis = javap[key]
    seen = []
    for seg in sm['segments']:
        origin = seg.get('origin', {})
        for fact in ([origin['primary']] if origin.get('primary') else []) + origin.get('derived', []):
            physical = fact.get('method', {})
            if physical.get('name') == item_name.get('raw') and physical.get('descriptor') == item_desc.get('raw'):
                bci = fact.get('bci')
                need(bci in actual_bcis, f'{fixture}: source-map BCI {bci} absent from javap {key}')
                seen.append(bci)
    need(seen, f'{fixture}: target method has no source-map facts tied to its actual method identity')
    need(set(seen)==actual_bcis, f'{fixture}: source-map physical instruction set differs from complete javap method')
    return {'identity': identity, 'member_ordinal': method['item']['index'], 'method_name_bytes': item_name.get('raw'),
            'descriptor_bytes': item_desc.get('raw'), 'checked_bci_count': len(seen), 'javap_instruction_count': len(actual_bcis)}

def classify_command(command):
    if command is None:
        return {'state': 'not_run'}
    if command.get('guard_stop') is not None:
        return {'state': 'guard_stopped', 'reason': command['guard_stop'], 'exit_code': command.get('exit_code')}
    code = command.get('exit_code')
    if code == 0:
        return {'state': 'exit0', 'exit_code': 0}
    if isinstance(code, int):
        raws=raw_for(command)
        combined=(raws['stdout']+b'\n'+raws['stderr']).decode('utf-8',errors='replace')
        lower=combined.lower()
        signatures=('unsupportedclassversionerror','class file has wrong version',
                    'has wrong version 55.0','major version 55.0','major version 55')
        signature=next((x for x in signatures if x in lower),None)
        if signature is None and re.search(r'类文件具有错误的版本\s+55\.0,\s*应为\s+52\.0',combined):
            signature='javac_zh_class_major55_expected52'
        if signature:
            return {'state':'environment_unavailable','exit_code':code,'signature':signature,
                    'stdout_sha256':command['streams']['stdout']['sha256'],
                    'stderr_sha256':command['streams']['stderr']['sha256']}
        return {'state': 'nonzero', 'exit_code': code}
    return {'state': 'missing_exit', 'exit_code': code}


def runtime_equal(runtime, original):
    if runtime is None or original is None:
        return None
    rr, ore = runtime.get('streams', {}), original.get('streams', {})
    return (runtime.get('exit_code') == original.get('exit_code') and
            rr.get('stdout', {}).get('sha256') == ore.get('stdout', {}).get('sha256') and
            rr.get('stderr', {}).get('sha256') == ore.get('stderr', {}).get('sha256'))


def source_api(api, idx, side, cli):
    cmd = api.get('command')
    need(isinstance(cmd, dict), 'missing class-source invocation record')
    command_matches_embedded(cmd, idx)
    validate_api_argv(cmd, api, side, cli)
    raws = raw_for(cmd)
    state = classify_command(cmd)
    try:
        doc = json.loads(raws['stdout'].decode('utf-8'))
    except Exception:
        return {'command_state': state, 'json_state': 'invalid_or_empty', 'doc': None}
    outcome = doc.get('outcome')
    if outcome == 'performed':
        need(isinstance(doc.get('text'), str) and doc['text'] and isinstance(doc.get('methods'), list),
             f"performed API missing full-source/methods: {cmd.get('label')}")
        return {'command_state': state, 'json_state': 'performed', 'doc': doc}
    return {'command_state': state, 'json_state': str(outcome or 'outcome_missing'), 'doc': doc}


def read_class_digest_manifest(path, fixture_rows):
    # Root’s frozen build record is the digest manifest. Recheck its oracle provenance,
    # invocation, raw output, and exact input set without importing or running BLAKE3.
    manifest = json.loads(path.read_text())
    need(manifest.get('schema') == 'switch-continuation-candidate-cli-build-root-v1',
         'unsupported root digest-oracle execution schema')
    need(manifest.get('status') == 'candidate-cli-and-class-digests-built', 'digest-oracle build did not complete')
    for key in ('runner','guard','scanner','plan','oracle_source','oracle_binary','cli'):
        verify_pin(manifest[key])
    library_pin=manifest['oracle_blake3_library']
    need(set(library_pin)=={'path','bytes','sha256'} and len(library_pin['sha256'])==64,
         'historical blake3 rlib pin malformed')
    if Path(library_pin['path']).exists():
        verify_pin(library_pin)
        library_state='present_and_verified'
    else:
        need(manifest.get('target_absent_after_clean') is True and '/target/' in library_pin['path'],
             'missing oracle rlib is not explained by the recorded post-build cargo clean')
        library_state='recorded_link_input_removed_by_cargo_clean'
    manifest['oracle_library_verification_state']=library_state
    need(manifest.get('target_absent_after_clean') is True, 'digest tool build did not record post-clean absence')
    plan_pin, _ = digest_file(PLAN)
    need({k:manifest['plan'][k] for k in ('path','bytes','sha256')} == plan_pin,
         'digest-oracle input plan differs from replay plan')
    commands=manifest.get('commands',[])
    need(len(commands)>=5, 'digest-oracle build commands incomplete')
    for cmd in commands:
        raw_for(cmd)
    rustc = next((c for c in commands if c.get('argv',[]) and c['argv'][0]=='rustc'),None)
    digest_run = next((c for c in commands if c.get('argv',[]) and Path(c['argv'][0]).resolve()==Path(manifest['oracle_binary']['path']).resolve()),None)
    need(rustc is not None and rustc.get('exit_code')==0 and rustc.get('guard_stop') is None, 'pinned digest oracle was not built successfully')
    need(digest_run is not None and digest_run.get('exit_code')==0 and digest_run.get('guard_stop') is None, 'pinned digest oracle invocation did not succeed')
    clean=next((c for c in commands if c.get('argv')==['cargo','clean']),None)
    need(clean is not None and clean.get('exit_code')==0 and clean.get('guard_stop') is None and clean['index']>digest_run['index'],
         'post-oracle cargo clean event missing; cannot explain absent rlib')
    argv=rustc['argv']
    need(str(Path(manifest['oracle_source']['path']).resolve()) in [str(Path(x).resolve()) for x in argv], 'rustc argv does not use pinned oracle source')
    need(any(x == 'blake3='+manifest['oracle_blake3_library']['path'] for x in argv if x.startswith('--extern ')) or
         any(x == '--extern' and i+1<len(argv) and argv[i+1]=='blake3='+manifest['oracle_blake3_library']['path'] for i,x in enumerate(argv)),
         'rustc argv does not bind the pinned blake3 library')
    need(str(Path(manifest['oracle_binary']['path']).resolve()) in [str(Path(x).resolve()) for x in argv], 'rustc argv does not write pinned oracle binary')
    expected = {str(Path(pin['path']).resolve()) for fixture in fixture_rows for pin in fixture.get('input_classes', [])}
    run_inputs=[str(Path(x).resolve()) for x in digest_run['argv'][1:]]
    need(len(run_inputs)==len(set(run_inputs)) and set(run_inputs)==expected, 'digest oracle argv does not cover exactly all replay input classes')
    entries=manifest.get('class_digests')
    need(isinstance(entries,list) and entries, 'class digest observations missing')
    by_path={}
    for row in entries:
        verify_pin(row)
        need(isinstance(row.get('blake3'),str) and re.fullmatch(r'[0-9a-f]{64}',row['blake3']),f"invalid BLAKE3 for {row.get('path')}")
        by_path[str(Path(row['path']).resolve())]=row
    need(len(by_path)==len(entries) and set(by_path)==expected, 'digest observations are not the exact eleven frozen class inputs')
    output_pin=digest_run['streams']['stdout']
    output=Path(output_pin['path']).read_bytes().decode('utf-8')
    parsed=[]
    for line in output.splitlines():
        fields=line.split('\t')
        need(len(fields)==3, 'digest oracle stdout row has unexpected format')
        parsed.append({'blake3':fields[0],'bytes':int(fields[1]),'path':fields[2]})
    recorded=sorted(({k:r[k] for k in ('blake3','bytes','path')} for r in entries),key=lambda x:x['path'])
    need(sorted(parsed,key=lambda x:x['path'])==recorded, 'execution digest observations differ from pinned oracle stdout')
    return manifest, by_path


COMPOUND_BUILD_SCHEMA = 'compound-condition-focused-and-cli-root-v2'
COMPOUND_BUILD_PATH = Path('/private/tmp/jarde-compound-condition-focused-and-cli-root-v1/execution.json').resolve()
REPO_ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
COMPOUND_SOURCE_PINS = {
    'Cargo.lock', 'Cargo.toml',
    'crates/jarde-cli/Cargo.toml', 'crates/jarde-cli/src/main.rs', 'crates/jarde-cli/src/task.rs',
    'crates/jarde-java/Cargo.toml', 'crates/jarde-java/src/build.rs', 'crates/jarde-java/src/emit.rs',
    'crates/jarde-java/src/lib.rs', 'crates/jarde-java/src/region.rs', 'crates/jarde-java/src/report.rs',
    'crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs',
    'crates/jarde-jvm/Cargo.toml', 'crates/jarde-query/Cargo.toml', 'crates/jarde-reader/Cargo.toml',
    'openspec/changes/recover-proved-compound-statement-conditions/results/complete-runtime-baseline-root-v1/observed/original-bytecode/CompoundStatementConditions.class',
    'src/class_source.rs', 'src/facade.rs', 'src/lib.rs',
}


def verify_candidate_build(path, replay_cli):
    build_path = Path(path).resolve()
    need(build_path == COMPOUND_BUILD_PATH, 'candidate build record is not the frozen compound CLI build execution')
    build_sha = sha_bytes(build_path.read_bytes())
    build = json.loads(build_path.read_text())
    need(build.get('schema') == COMPOUND_BUILD_SCHEMA, 'unsupported compound candidate build schema')
    need(build.get('status') == 'candidate-cli-built', 'compound candidate CLI build did not complete')
    need(build.get('cli_is_unaccepted_candidate') is True, 'compound build does not identify its CLI as unaccepted')
    cli = build.get('cli')
    need(isinstance(cli, dict) and set(cli) == {'path', 'bytes', 'sha256', 'mode'},
         'compound candidate CLI pin malformed')
    verify_pin(cli)
    need(cli['mode'] == 0o555 and (Path(cli['path']).stat().st_mode & 0o777) == 0o555,
         'compound candidate CLI must be pinned executable mode 0555')
    need({k: replay_cli.get(k) for k in ('path', 'bytes', 'sha256')} ==
         {k: cli.get(k) for k in ('path', 'bytes', 'sha256')},
         'full-class replay CLI does not match the separately pinned compound candidate build')

    before, after = build.get('product_pins_before'), build.get('product_pins_after')
    need(isinstance(before, dict) and isinstance(after, dict), 'compound product source pin maps missing')
    need(set(before) == COMPOUND_SOURCE_PINS and set(after) == COMPOUND_SOURCE_PINS and len(before) == 19,
         'compound candidate build must contain the exact 19-file product/source pin set')
    need(before == after, 'compound product/source pins changed during candidate build')
    for rel in sorted(COMPOUND_SOURCE_PINS):
        target = (REPO_ROOT / rel).resolve()
        need(target.is_relative_to(REPO_ROOT), f'compound source pin escapes repository: {rel}')
        actual = sha_bytes(target.read_bytes())
        need(actual == before[rel], f'compound source pin differs from live file: {rel}')

    commands = build.get('commands')
    need(isinstance(commands, list) and len(commands) == 5, 'compound build command envelope must contain five commands')
    expected_argv = [
        ['cargo', 'fmt', '--all', '--', '--check'],
        ['cargo', 'test', '-p', 'jarde-java', '--lib', '--locked', 'region::tests::', '--', '--nocapture'],
        ['cargo', 'test', '-p', 'jarde-java', '--test', 'p3_conditional_switch_fallthrough', '--locked', '--', '--nocapture'],
        ['cargo', 'build', '-p', 'jarde-cli', '--locked'],
        ['cargo', 'clean'],
    ]
    for i, (cmd, argv) in enumerate(zip(commands, expected_argv)):
        need(cmd.get('index') == i and cmd.get('argv') == argv,
             f'compound build command {i} argv/index differs from frozen successful build')
        need(Path(cmd.get('cwd', '')).resolve() == REPO_ROOT, f'compound build command {i} cwd differs from repository')
        need(cmd.get('exit_code') == 0 and cmd.get('guard_stop') is None,
             f'compound build command {i} did not finish successfully without a guard stop')
        raw_for(cmd)
    need(build.get('target_absent_after_clean') is True,
         'compound candidate build did not record target absence after cargo clean')
    # Require the test counts from the pinned raw streams, not just runner summary fields.
    for index, count in ((1, 17), (2, 6)):
        raw = raw_for(commands[index])
        stdout = re.sub(r'\x1b\[[0-9;]*m', '', raw['stdout'].decode('utf-8', errors='strict'))
        summary = re.findall(r'^test result: ok\. (\d+) passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out; finished in [^\n]+$',
                             stdout, flags=re.MULTILINE)
        need(summary == [str(count)], f'compound build command {index} raw test summary is not exactly {count}/0')
    return {'path': str(build_path), 'sha256': build_sha, 'cli': cli,
            'product_source_pin_count': 19, 'successful_command_count': 5,
            'region_test_count': 17, 'public_test_count': 6, 'target_absent_after_clean': True}

def baseline_rows():
    pins = [
      {'path': '/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/full-replay-root-v1/execution.json',
       'bytes': 288964, 'sha256': '04ee982bad32e216dc1908fddeec588016b5c209f1d0b1f64e0683689501f16c'},
      {'path': '/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-remaining-root-v1/full-replay-root-v1/execution.json',
       'bytes': 254360, 'sha256': '68dedcf061eddaf36036265a703894f3d2584a20379b03b130d181a5355872d1'},
    ]
    rows = {}
    for pin in pins:
        verify_pin(pin)
        baseline = json.loads(Path(pin['path']).read_text())
        need(baseline.get('status') in ('observed_complete_source_comparison', 'observed_complete_source_comparison_pending_independent_acceptance'), f"historical baseline status unexpected: {pin['path']}")
        # Verify all historical raw streams too; these remain oracle-only data.
        for cmd in baseline.get('commands', []):
            legacy_raw_for(cmd)
        for case in baseline.get('cases', []):
            fixture = case['case'].removesuffix('.test')
            if case['kind'] in ('jadx', 'jarde-default', 'jarde-all'):
                rows[(fixture, case['kind'])] = case
    return rows, pins


def validate_runtime_gate(side_name, runtime, skip_reason, compile_state, classes_match,
                          probe_state, api_statuses, original_state):
    api_ok = side_name == 'jadx' or all(
        a.get('command',{}).get('state') == 'exit0' and a.get('json') == 'performed'
        for a in api_statuses)
    probe_ok = probe_state.get('state') == 'exit0'
    original_ran = original_state.get('state') != 'skipped'
    compile_ok = compile_state.get('state') == 'exit0'
    reasons=[]
    if not compile_ok: reasons.append('compile_failed_or_stopped')
    if not classes_match: reasons.append('compiled_class_set_differs_from_original')
    if not probe_ok: reasons.append('probe_compile_failed')
    if not api_ok: reasons.append('class_source_api_not_all_successful')
    if not original_ran: reasons.append('original_probe_runtime_skipped')
    eligible = not reasons
    if runtime is not None:
        need(eligible, f'{side_name}: runtime executed despite unmet runner gate {reasons}')
        need(skip_reason is None, f'{side_name}: runtime was run but skip reasons are recorded')
    else:
        need(bool(reasons), f'{side_name}: runtime skipped although all runner gates passed')
        need(isinstance(skip_reason,list) and set(skip_reason)==set(reasons),
             f'{side_name}: skip reason does not exactly match observed gate failures: expected {reasons}, got {skip_reason}')
    return {'eligible':eligible,'failed_gates':reasons}


def expected_pre_run_pins(execution, plan):
    runner_pin=execution['runner']
    verify_pin(runner_pin)
    # Read the pinned runner as data only to obtain its frozen historical JUnit pin literals.
    tree=ast.parse(Path(runner_pin['path']).read_text(),filename=runner_pin['path'])
    historical=None
    for node in tree.body:
        if isinstance(node,(ast.Assign,ast.AnnAssign)):
            targets=node.targets if isinstance(node,ast.Assign) else [node.target]
            if any(isinstance(t,ast.Name) and t.id=='HISTORICAL_JUNIT' for t in targets):
                historical=ast.literal_eval(node.value)
                break
    need(isinstance(historical,dict) and set(historical)=={'original-five','remaining-five'},
         'pinned runner historical JUnit pins are unreadable or unexpected')
    expected=[plan_pin(PLAN),runner_pin,execution['cli'],execution['python_runtime']]
    shared=plan['shared_pins']
    for group in ('old_full_replay_runner','remaining_five_full_replay_runner',
                  'remaining_five_direct_harness','old_observer','remaining_observer',
                  'old_runtime_probe','remaining_runtime_probe'):
        expected.append(shared[group])
    for group in ('test_sdk_jars','jadx_product_jars','helper_class_files'):
        expected.extend(shared[group])
    for batch in historical.values():
        for pin in batch.values(): expected.append(pin)
    for jdk in ('jdk8_tools','jdk23_tools'):
        for tool in shared[jdk].values():
            expected.append(tool)
            resolved=Path(tool['path']).resolve()
            resolved_pin,_=digest_file(resolved)
            need(resolved_pin['sha256']==tool['sha256'],f'resolved JDK tool differs from frozen plan: {tool["path"]}')
            expected.append(resolved_pin)
    for fixture in plan['fixtures']:
        expected.extend(fixture['pins'])
    return expected


def verify_workspace_envelope(execution, plan):
    expected=expected_pre_run_pins(execution,plan)
    listed=execution.get('pre_run_pins')
    need(isinstance(listed,list), 'pre_run_pins missing')
    key=lambda p:canonical({k:p[k] for k in ('path','bytes','sha256')})
    need(collections.Counter(map(key,listed))==collections.Counter(map(key,expected)),
         'pre-run pins are not the exact runner/plan/JDK/SDK/helper/fixture manifest')
    for pin in listed: verify_pin(pin)
    need(execution.get('post_run_pin_check')=='passed','runner did not record symmetric post-run pin pass')
    # Rehash the same exact set after execution; the result record alone is not trusted.
    for pin in expected: verify_pin(pin)
    helper_root=Path(execution['classpath'][0]).resolve()
    helper_source_root=Path('/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/full-replay-root-v1/helpers')
    expected_copied=[]
    for pin in plan['shared_pins']['helper_class_files']:
        src=Path(pin['path'])
        rel=src.relative_to(helper_source_root)
        if rel.as_posix().startswith(('jadx/tests/integration/switches/','cf12capture/')): continue
        dest=helper_root/rel
        expected_copied.append({'path':str(dest),'bytes':pin['bytes'],'sha256':pin['sha256']})
    copied=execution.get('copied_helpers')
    need(isinstance(copied,list) and len(copied)==len(expected_copied), 'copied helper class inventory count differs from plan')
    need(sorted(copied,key=lambda x:x['path'])==sorted(expected_copied,key=lambda x:x['path']),
         'copied helper files differ from exact pinned helper source set')
    for pin in copied: verify_pin(pin)
    forbidden_prefixes=('jadx/tests/integration/switches/','cf12capture/')
    need(all(not Path(p['path']).relative_to(helper_root).as_posix().startswith(forbidden_prefixes) for p in copied),
         'target/observer classes present in copied helper overlay')
    sdk=[x for x in plan['shared_pins']['test_sdk_jars']]
    product=[x for x in plan['shared_pins']['jadx_product_jars']]
    expected_jars=product+sdk
    jar_rows=execution.get('classpath_jars')
    need(isinstance(jar_rows,list) and len(jar_rows)==len(expected_jars), 'classpath jar inventory count differs from frozen pins')
    need([r['pin'] for r in jar_rows]==expected_jars, 'classpath jar pin order/set differs from frozen SDK/product set')
    for row,pin in zip(jar_rows,expected_jars):
        verify_pin(row['pin'])
        with zipfile.ZipFile(pin['path']) as archive:
            names=archive.namelist()
        classes=[n for n in names if n.endswith('.class')]
        forbidden=[n for n in classes if n.startswith(forbidden_prefixes)]
        need(row.get('class_entries')==len(classes) and row.get('forbidden_target_entries')==forbidden and not forbidden,
             f"classpath jar inventory mismatch/forbidden target class: {pin['path']}")
    expected_cp=[str(helper_root),*[p['path'] for p in expected_jars]]
    need(execution.get('classpath')==expected_cp, 'runtime classpath differs from audited helper/jar inventory')
    return {'pre_run_pin_count':len(listed),'copied_helper_count':len(copied),
            'classpath_jar_count':len(jar_rows),'post_run_rehash':'passed'}


def plan_pin(path):
    actual,_=digest_file(path)
    return actual


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--execution', required=True, type=Path)
    ap.add_argument('--class-digests', required=True, type=Path)
    ap.add_argument('--candidate-build', required=True, type=Path)
    args = ap.parse_args()
    execution_path = args.execution.resolve()
    execution = json.loads(execution_path.read_text())
    plan = json.loads(PLAN.read_text())
    need(execution.get('schema') == 'cf12-ten-fixture-full-class-replay-candidate-luna-v1', 'unsupported execution schema')
    need(execution.get('cf12_complete') is False, 'execution claims CF12 complete; refuse self-certification')
    need(execution.get('status') == 'observed_pending_root_independent_acceptance', 'execution status is not observed/pending')
    verify_pin(execution['cli'])
    need(execution.get('cli_mode') == 0o555 and (Path(execution['cli']['path']).stat().st_mode & 0o777) == 0o555, 'CLI mode pin mismatch')
    candidate_build = verify_candidate_build(args.candidate_build, execution['cli'])
    workspace_envelope=verify_workspace_envelope(execution,plan)
    fixture_rows = execution.get('fixtures', [])
    need(len(plan['fixtures']) == 10 and len(fixture_rows) == 10, 'need ten planned and observed fixture rows')
    idx = command_index(execution)
    digest_manifest_sha=sha_bytes(args.class_digests.read_bytes())
    digest_manifest, digests = read_class_digest_manifest(args.class_digests.resolve(), fixture_rows)
    need(execution.get('plan')==plan_pin(PLAN),'replay top-level plan pin differs from pinned plan')
    # The class digests are historical oracle evidence from a separate CLI build.
    # Their CLI pin remains verified by read_class_digest_manifest, but is deliberately
    # not equated with the newly built compound candidate CLI.
    need(digest_manifest.get('cli_is_unaccepted_candidate') is True,'digest source does not identify candidate CLI as unaccepted')
    fixtures = {f['fixture']: f for f in fixture_rows}
    need(len(fixtures) == 10 and set(fixtures) == {f['fixture'] for f in plan['fixtures']}, 'fixture set differs from fixed plan')

    # A complete observation envelope includes every expected profile and every attempted invocation,
    # even when the real result is rejection, compilation failure, runtime failure, or a guarded stop.
    observed = {'fixture_count': 10, 'profile_count': 20, 'side_count': 80,
                'probe_compile_states': {}, 'compile_states': {}, 'runtime_states': {}, 'api_states': {},
                'class_set_differences': [], 'baseline_comparisons': [],
                'default_all_api_comparisons': []}
    referenced = []
    for jdk in JDKS:
        for family in ('original-five', 'remaining-five'):
            rec = execution['probe_compilation'][jdk][family]
            referenced.append(rec['compile'])
            source_pin=plan['shared_pins']['old_runtime_probe' if family=='original-five' else 'remaining_runtime_probe']
            verify_pin(rec['source'])
            need(rec['source']['bytes']==source_pin['bytes'] and rec['source']['sha256']==source_pin['sha256'],
                 f'{jdk}/{family}: copied probe source differs from pinned helper source')
            expected_probe_name='Cf12RuntimeProbe' if family=='original-five' else 'Cf12RemainingRuntimeProbe'
            need(rec.get('class_name')==expected_probe_name, f'{jdk}/{family}: probe class name mismatch')
            tool = plan['shared_pins'][jdk]['javac']
            check_tool_argv(rec['compile'], tool, 'javac')
            target = Path(rec['directory']).resolve()
            expected_argv = [str(Path(tool['path']).resolve()), '-encoding', 'UTF-8', '-source', '8', '-target', '8',
                '-proc:none', '-classpath', os.pathsep.join(execution['classpath']), '-sourcepath', str(target/'empty-sourcepath'),
                '-d', str(target), str(Path(rec['source']['path']).resolve())]
            need(rec['compile']['argv'] == expected_argv and Path(rec['compile']['cwd']).resolve() == Path('/Users/lordcasser/workspace/projects/jarde').resolve(),
                 f'{jdk}/{family}: exact probe compile invocation mismatch')
            expected_probe = 'Cf12RuntimeProbe.class' if family == 'original-five' else 'Cf12RemainingRuntimeProbe.class'
            need(isinstance(rec.get('classes'),list), f'{jdk}/{family}: actual probe class set missing')
            if rec['compile'].get('exit_code') == 0 and rec['compile'].get('guard_stop') is None:
                need(rec['classes'] == [expected_probe], f'{jdk}/{family}: successful probe class set mismatch')
            observed['probe_compile_states'][f'{jdk}/{family}']=classify_command(rec['compile'])
    details = {}
    for fixture_plan in plan['fixtures']:
        name = fixture_plan['fixture']
        row = fixtures[name]
        expected_classes = sorted((p for p in fixture_plan['pins'] if '/input/' in p['path'] and p['path'].endswith('.class')), key=lambda x:x['path'])
        need(sorted(row.get('input_classes', []), key=lambda x:x['path']) == expected_classes and expected_classes,
             f'{name}: pinned complete original class set missing/different')
        for pin in expected_classes: verify_pin(pin)
        expected_jadx = sorted((p for p in fixture_plan['pins'] if '/jadx-source/' in p['path'] and p['path'].endswith('.java')), key=lambda x:x['path'])
        need(sorted(row.get('jadx_full_sources', []), key=lambda x:x['path']) == expected_jadx and expected_jadx,
             f'{name}: pinned full JADX source set missing/different')
        for pin in expected_jadx: verify_pin(pin)
        profiles = {p['jdk']:p for p in row.get('jdk_profiles', [])}
        need(set(profiles) == set(JDKS), f'{name}: missing JDK profile')
        details[name] = {}
        for jdk in JDKS:
            profile = profiles[jdk]
            sides = profile.get('sides', {})
            need(set(sides) == set(SIDES), f'{name}/{jdk}: four-side observation envelope incomplete')
            original = sides['original']
            tools = plan['shared_pins'][jdk]
            family = 'original-five' if name in {'TestSwitch','TestSwitchNoDefault','TestSwitchLabels','TestSwitchFallThrough','TestSwitchWithFallThroughCase'} else 'remaining-five'
            probe_dir = Path(execution['probe_compilation'][jdk][family]['directory']).resolve()
            original_dir = (execution_path.parent / name / jdk / 'original').resolve()
            class_dir = original_dir / 'classes'
            probe_class = 'Cf12RuntimeProbe' if family == 'original-five' else 'Cf12RemainingRuntimeProbe'
            target_class = 'jadx.tests.integration.switches.' + name + '$TestCls'
            if original.get('runtime') is not None:
                referenced.append(original['runtime'])
                cp = os.pathsep.join([str(probe_dir),str(class_dir),*execution['classpath']])
                expected_argv=[str(Path(tools['java']['path']).resolve()),'-Xverify:all','-cp',cp,probe_class,target_class,name]
                need(original['runtime']['argv']==expected_argv and Path(original['runtime']['cwd']).resolve()==original_dir,
                     f'{name}/{jdk}: original runtime exact JDK/target invocation mismatch')
                check_tool_argv(original['runtime'],tools['java'],'java')
                original_state = classify_command(original['runtime'])
            else:
                original_state = {'state':'skipped','reason':original.get('runtime_skip_reason')}
                probe_state=observed['probe_compile_states'][f'{jdk}/{family}']
                need(probe_state.get('state')!='exit0' and original_state['reason']=='probe_compile_failed',
                     f'{name}/{jdk}: original runtime skip reason disagrees with probe result')
            observed['runtime_states'][f'{name}/{jdk}/original'] = original_state
            original_classes = original.get('classes')
            actual_original=sorted(p.relative_to(class_dir).as_posix() for p in class_dir.rglob('*.class'))
            need(original_classes==actual_original,f'{name}/{jdk}: recorded original class set differs from disk')
            for inp in expected_classes:
                copied=class_dir/'jadx/tests/integration/switches'/Path(inp['path']).name
                need(copied.read_bytes()==Path(inp['path']).read_bytes(),f'{name}/{jdk}: copied original class bytes differ from input')
            expected_original_classes=sorted('jadx/tests/integration/switches/'+Path(p['path']).name for p in expected_classes)
            need(isinstance(original_classes, list) and original_classes==expected_original_classes,
                 f'{name}/{jdk}: original actual class set differs from exact pinned input classes')
            profile_detail = {'original': {'runtime': original_state, 'classes': original_classes}, 'sides': {}}
            for side_name in SIDES[1:]:
                side = sides[side_name]
                compile_rec = side.get('compile')
                source_pins=side.get('sources',[])
                need(isinstance(source_pins,list),f'{name}/{jdk}/{side_name}: source file pin list missing')
                if side_name=='jadx':
                    need(bool(source_pins) and compile_rec is not None,f'{name}/{jdk}: JADX compile invocation missing')
                elif bool(source_pins) != (compile_rec is not None):
                    raise VerifyError(f'{name}/{jdk}/{side_name}: compile invocation/source set envelope mismatch')
                if compile_rec is not None:
                    referenced.append(compile_rec)
                    javac=plan['shared_pins'][jdk]['javac']
                    check_tool_argv(compile_rec,javac,'javac')
                    sd=Path(compile_rec['cwd']).resolve()
                    expected_compile=[str(Path(javac['path']).resolve()),'-encoding','UTF-8','-g','-source','8','-target','8',
                        '-proc:none','-classpath',os.pathsep.join(execution['classpath']),'-sourcepath',str(sd/'empty-sourcepath'),
                        '-d',str(sd/'classes'),*sorted(p['path'] for p in side.get('sources',[]))]
                    need(compile_rec['argv']==expected_compile, f'{name}/{jdk}/{side_name}: exact compile argv mismatch')
                compile_state = classify_command(compile_rec)
                observed['compile_states'][f'{name}/{jdk}/{side_name}'] = compile_state
                for source_pin in source_pins: verify_pin(source_pin)
                if side_name=='jadx':
                    need([p['sha256'] for p in source_pins]==[p['sha256'] for p in row['jadx_full_sources']],
                         f'{name}/{jdk}: copied JADX source bytes differ from frozen full-source set')
                compiled = side.get('compiled_classes')
                generated_class_dir=execution_path.parent/name/jdk/side_name/'classes'
                actual_generated=sorted(p.relative_to(generated_class_dir).as_posix() for p in generated_class_dir.rglob('*.class'))
                need(compiled==actual_generated,f'{name}/{jdk}/{side_name}: recorded generated class set differs from disk')
                need(isinstance(compiled, list), f'{name}/{jdk}/{side_name}: actual compiled class set missing')
                set_match = compiled == original_classes
                if not set_match:
                    observed['class_set_differences'].append({'fixture':name,'jdk':jdk,'side':side_name,
                                                              'original':original_classes,'generated':compiled})
                api_statuses = []
                docs = {}
                api_rows = side.get('class_source_api', [])
                if side_name in ('jarde-default','jarde-all'):
                    api_paths = [str(Path(a.get('input', {}).get('path', '')).resolve()) for a in api_rows]
                    expected_paths = [str(Path(p['path']).resolve()) for p in expected_classes]
                    need(len(api_paths) == len(set(api_paths)) and set(api_paths) == set(expected_paths),
                         f'{name}/{jdk}/{side_name}: class-source inputs differ from exact fixture class set')
                for api in api_rows:
                    referenced.append(api['command'])
                    if side_name in ('jarde-default','jarde-all'):
                        api_obs = source_api(api, idx, side_name, execution['cli']['path'])
                        expected_internal = 'jadx.tests.integration.switches.' + Path(api['input']['path']).stem
                        need(api.get('class') == expected_internal, f'{name}/{jdk}/{side_name}: class-source internal name mismatch')
                        verify_pin(api['input'])
                        api_statuses.append({'command':api_obs['command_state'],'json':api_obs['json_state']})
                        if api_obs['doc'] is not None and api_obs['json_state'] == 'performed':
                            pin = api['input']
                            dentry = digests[str(Path(pin['path']).resolve())]
                            for method in api_obs['doc']['methods']:
                                identity = method.get('item', {}).get('identity')
                                verify_source_map(method, name, identity, dentry['blake3'], dentry['bytes'])
                            if api.get('source_written'):
                                source_pin=api.get('source')
                                need(isinstance(source_pin,dict),f'{name}/{jdk}/{side_name}: written source pin missing')
                                verify_pin(source_pin)
                                need(Path(source_pin['path']).read_bytes()==api_obs['doc']['text'].encode('utf-8'),
                                     f'{name}/{jdk}/{side_name}: written source differs from raw class-source text')
                                need(source_pin in side.get('sources',[]),f'{name}/{jdk}/{side_name}: written source absent from compile inputs')
                            docs[str(Path(pin['path']).resolve())] = api_obs['doc']
                    else:
                        command_matches_embedded(api['command'], idx)
                        api_obs = {'json_state':'not_applicable', 'command_state':classify_command(api['command'])}
                        api_statuses.append({'command':api_obs['command_state'],'json':'not_applicable'})
                # Exact call envelope: original/JADX compile once; Jarde invokes class-source for every input class.
                if side_name in ('jarde-default','jarde-all'):
                    need(len(side.get('class_source_api', [])) == len(expected_classes),
                         f'{name}/{jdk}/{side_name}: class-source call envelope incomplete')
                else:
                    need(len(side.get('class_source_api', [])) == 0,
                         f'{name}/{jdk}/{side_name}: unexpected class-source calls')
                runtime = side.get('runtime')
                if runtime is not None:
                    referenced.append(runtime)
                    runtime_state = classify_command(runtime)
                    command_matches_embedded(runtime, idx)
                    java=plan['shared_pins'][jdk]['java']
                    sd=Path(runtime['cwd']).resolve()
                    probe_dir=Path(execution['probe_compilation'][jdk][family]['directory']).resolve()
                    generated_dir=sd/'classes'
                    cp=os.pathsep.join([str(probe_dir),str(generated_dir),*execution['classpath']])
                    expected_runtime=[str(Path(java['path']).resolve()),'-Xverify:all','-cp',cp,probe_class,target_class,name]
                    need(runtime['argv']==expected_runtime,f'{name}/{jdk}/{side_name}: exact runtime argv mismatch')
                    check_tool_argv(runtime,java,'java')
                else:
                    runtime_state = {'state':'skipped', 'reason':side.get('runtime_skip_reason')}
                    need(runtime_state['reason'], f'{name}/{jdk}/{side_name}: skipped runtime lacks reason')
                probe_state=observed['probe_compile_states'][f'{jdk}/{family}']
                gate=validate_runtime_gate(side_name,runtime,side.get('runtime_skip_reason'),compile_state,set_match,
                                           probe_state,api_statuses,original_state)
                runtime_state['gate']=gate
                actual_equal = runtime_equal(runtime, original.get('runtime'))
                if side.get('runtime_equal_original_stdout_stderr') is not None:
                    need(side.get('runtime_equal_original_stdout_stderr') == actual_equal,
                         f'{name}/{jdk}/{side_name}: stored runtime equality differs from actual raw records')
                observed['runtime_states'][f'{name}/{jdk}/{side_name}'] = runtime_state
                observed['api_states'][f'{name}/{jdk}/{side_name}'] = api_statuses
                profile_detail['sides'][side_name] = {
                    'compile':compile_state, 'api':api_statuses, 'compiled_classes':compiled,
                    'class_set_matches_original':set_match, 'runtime':runtime_state,
                    'runtime_equal_original_stdout_stderr':actual_equal,
                    'full_source_documents':len(docs)}
                if side_name in ('jarde-default','jarde-all'):
                    profile_detail['sides'][side_name]['docs'] = docs
            # Record exact all/default comparison when both outputs performed, preserving deltas otherwise.
            d=profile_detail['sides']['jarde-default']['docs']; a=profile_detail['sides']['jarde-all']['docs']
            cmp={'fixture':name,'jdk':jdk,'comparable_inputs':sorted(set(d)&set(a)),
                 'missing_default':sorted(set(a)-set(d)),'missing_all':sorted(set(d)-set(a)), 'equal':None}
            if set(d)==set(a) and d:
                equal=True
                for key in d:
                    left,right=d[key],a[key]
                    if left.get('text') != right.get('text') or len(left.get('methods',[])) != len(right.get('methods',[])):
                        equal=False; continue
                    for lm,rm in zip(left['methods'],right['methods']):
                        lr=lm.get('outcome',{}).get('report',{}); rr=rm.get('outcome',{}).get('report',{})
                        if canonical(lm.get('item')) != canonical(rm.get('item')) or lr.get('text') != rr.get('text') or canonical(lr.get('source_map')) != canonical(rr.get('source_map')):
                            equal=False
                cmp['equal']=equal
            observed['default_all_api_comparisons'].append(cmp)
            details[name][jdk]=profile_detail

    # Require every raw command in the execution list to be referenced exactly once by the observation envelope.
    labels=[r.get('label') for r in referenced]
    need(None not in labels and len(labels)==len(set(labels)), 'duplicate or missing invocation reference in observation envelope')
    commands=execution.get('commands',[])
    need(len(commands)==len(referenced) and len(idx)==len(commands), 'extra/missing/duplicate raw command row')
    need([r.get('index') for r in commands]==list(range(1,len(commands)+1)), 'command indices are not contiguous/ordered')
    by_label={r['label']:r for r in commands}
    for ref in referenced:
        need(ref['label'] in by_label and canonical(ref)==canonical(by_label[ref['label']]),
             f"invocation record does not match exact raw row: {ref.get('label')}")

    # Independently inspect available target-method source origins against frozen javap instructions.
    target_specs={'TestSwitch2':('test','(I)V'),
                  'TestSwitchWithFallThroughCase2':('test','(IZZ)Ljava/lang/String;')}
    target_origins={}
    for fixture,(method_name,descriptor) in target_specs.items():
        row=fixtures[fixture]
        input_pin=next(p for p in row['input_classes'] if Path(p['path']).name.endswith('$TestCls.class'))
        digest=digests[str(Path(input_pin['path']).resolve())]
        javap=find_javap(execution,fixture,input_pin['path'])
        target_origins[fixture]={}
        for jdk in JDKS:
            prof=details[fixture][jdk]
            for side_name in ('jarde-default','jarde-all'):
                docs=prof['sides'][side_name]['docs']
                doc=docs.get(str(Path(input_pin['path']).resolve()))
                key=f'{jdk}/{side_name}'
                if doc is None:
                    target_origins[fixture][key]={'state':'unavailable_due_to_class_source_outcome'}
                    continue
                found=verify_changed_method(doc,fixture,method_name,descriptor,digest['blake3'],digest['bytes'],javap)
                target_origins[fixture][key]={'state':'verified_against_actual_javap','identity':found['identity'],
                    'member_ordinal':found['member_ordinal'],'checked_bci_count':found['checked_bci_count'],
                    'javap_instruction_count':found['javap_instruction_count']}

    # The fixed prior ten-fixture Jarde replay is an oracle for exact before/after outcomes only.
    old, baseline_pins = baseline_rows()
    for fixture in fixtures:
        if (fixture, 'jadx') not in old: continue
        profile=next(p for p in fixtures[fixture]['jdk_profiles'] if p['jdk']=='jdk23_tools')
        for side_name in ('jadx','jarde-default','jarde-all'):
            base=old.get((fixture,side_name))
            if base is None: continue
            current=profile['sides'][side_name]
            cc=current.get('compile'); rt=current.get('runtime')
            current_sources=sorted(p['sha256'] for p in current.get('sources',[]))
            base_sources=sorted(p['sha256'] for p in base.get('source_files',[]))
            now={'compile_exit':cc.get('exit_code') if cc else None,
                 'runtime_exit':rt.get('exit_code') if rt else None,
                 'runtime_raw_equal_original':runtime_equal(rt, profile['sides']['original'].get('runtime')),
                 'source_sha256':current_sources}
            before={'compile_exit':base.get('compile_exit'),'runtime_exit':base.get('runtime_exit'),
                    'runtime_raw_equal_original':base.get('runtime_raw_equal_original'),'source_sha256':base_sources}
            observed['baseline_comparisons'].append({'fixture':fixture,'side':side_name,'baseline':before,'observed':now,
                                                      'recorded_fields_equal':canonical(before)==canonical(now),
                                                      'historical_source_evidence':'recorded_sha256_only_source_bytes_not_reverified'})

    # Specific milestone results are computed from observed facts; a missing target is a false milestone, not a verifier failure.
    def positive(name):
        for jdk in JDKS:
            profile=details[name][jdk]
            original=profile['original']
            for side_name in ('jarde-default','jarde-all'):
                side=profile['sides'][side_name]
                if side['compile']['state']!='exit0' or not side['class_set_matches_original']:
                    return False
                if side['runtime']['state']!='exit0' or side['runtime_equal_original_stdout_stderr'] is not True:
                    return False
                if target_origins[name][f'{jdk}/{side_name}']['state']!='verified_against_actual_javap':
                    return False
        return True
    need(len(observed['default_all_api_comparisons']) == 20 and
         all(row.get('equal') is True and not row.get('missing_default') and not row.get('missing_all')
             for row in observed['default_all_api_comparisons']),
         'default/all identity, text, and source-map comparisons must all be present and equal across 20 profiles')
    ft2_positive=positive('TestSwitchWithFallThroughCase2')
    switch2_restored=positive('TestSwitch2')
    need(ft2_positive, 'FT2 control did not compile/run identically and verify all physical origins')
    need(switch2_restored, 'Switch2 did not compile/run identically and verify all physical origins')
    for fixture, expected_count in (('TestSwitch2', 78), ('TestSwitchWithFallThroughCase2', 90)):
        need(set(target_origins[fixture]) == {f'{jdk}/{side}' for jdk in JDKS for side in ('jarde-default','jarde-all')},
             f'{fixture}: missing JDK/default/all physical-origin profile')
        for key, origin in target_origins[fixture].items():
            need(origin.get('state') == 'verified_against_actual_javap' and
                 origin.get('javap_instruction_count') == expected_count,
                 f'{fixture}/{key}: complete physical BCI set must equal the {expected_count}-instruction javap method')
    profile_summary={}
    for fname, fprofiles in details.items():
        profile_summary[fname]={}
        for jdk, profile in fprofiles.items():
            profile_summary[fname][jdk]={'original':profile['original'],'sides':{}}
            for side_name, side in profile['sides'].items():
                profile_summary[fname][jdk]['sides'][side_name]={k:v for k,v in side.items() if k!='docs'}
    need(sha_bytes(args.class_digests.read_bytes())==digest_manifest_sha,'class digest manifest changed during verification')
    need(verify_candidate_build(args.candidate_build, execution['cli']) == candidate_build,
         'compound candidate build/source pins changed during verification')
    report={'schema':'cf12-ten-fixture-compound-independent-verifier-luna-v8',
      'execution':str(execution_path),'execution_sha256':sha_bytes(execution_path.read_bytes()),
      'status':'observed_pending_root_independent_acceptance','cf12_complete':False,
      'complete_milestone':bool(ft2_positive and switch2_restored),
      'target_results':{'FT2_positive':ft2_positive,'Switch2_restored':switch2_restored},
      'observation_envelope':observed,'profile_observations':profile_summary,'target_source_origins':target_origins,
      'workspace_pin_envelope':workspace_envelope,
      'class_digest_manifest':{'path':str(args.class_digests.resolve()),'sha256':digest_manifest_sha,
                               'oracle_provenance':{k:digest_manifest[k] for k in ('oracle_source','oracle_blake3_library','oracle_binary','cli','plan','runner','guard','scanner')},'oracle_library_state':digest_manifest['oracle_library_verification_state']},
      'compound_candidate_build':candidate_build,
      'fixed_baseline_pins':baseline_pins,
      'baseline_scope':'historical command raw streams are rehashed; historical source_files provide only recorded SHA-256 values (no source byte/length pins), so comparisons cover recorded source hashes and outcomes only',
      'failure_counts':{
        'compile_nonzero':sum(v['state']=='nonzero' for v in observed['compile_states'].values()),
        'compile_environment_unavailable':sum(v['state']=='environment_unavailable' for v in observed['compile_states'].values()),
        'compile_guard_stopped':sum(v['state']=='guard_stopped' for v in observed['compile_states'].values()),
        'compile_not_run':sum(v['state']=='not_run' for v in observed['compile_states'].values()),
        'api_nonperformed':sum(1 for rows in observed['api_states'].values() for a in rows if a.get('json') not in ('performed','not_applicable')),
        'runtime_nonzero':sum(v['state']=='nonzero' for v in observed['runtime_states'].values()),
        'runtime_environment_unavailable':sum(v['state']=='environment_unavailable' for v in observed['runtime_states'].values()),
        'runtime_guard_stopped':sum(v['state']=='guard_stopped' for v in observed['runtime_states'].values()),
        'runtime_skipped':sum(v['state']=='skipped' for v in observed['runtime_states'].values()),
        'class_set_different':len(observed['class_set_differences'])},
      'note':'Acceptance requires a pinned compound candidate build, all 20 default/all comparisons, and positive Switch2 plus FT2 physical-origin/runtime controls. Environment failures remain classified from their raw streams.'}
    print(json.dumps(report,ensure_ascii=False,indent=2))

if __name__ == '__main__':
    try:
        main()
    except (VerifyError, KeyError, TypeError, ValueError, OSError) as exc:
        print(f'VERIFICATION FAILED: {exc}', file=sys.stderr)
        raise SystemExit(1)
