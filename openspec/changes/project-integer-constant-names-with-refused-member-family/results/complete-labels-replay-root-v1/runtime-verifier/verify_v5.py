#!/usr/bin/env python3
"""Strict read-only verifier for the observed v5 CF12 replay; launches no tools."""
import argparse, hashlib, json
from pathlib import Path

ROOT_CLASS = 'TestSwitchLabels$TestCls'
INNER_CLASS = ROOT_CLASS + '$Inner'
PACKAGE_PATH = 'jadx/tests/integration/switches/'
BINARY = 'jadx.tests.integration.switches.' + ROOT_CLASS
INPUTS = {
    ROOT_CLASS: (753, '4997ca33261d0585a1008dd860131c5ed3f2438206209bb7acb82c90eb743d22'),
    INNER_CLASS: (744, '2d6221bb75ba81cba428c32e3139038982c4078c80a1af99532c3a45acd54a99'),
}
MODES = ('default', 'all')
JDKS = ('jdk8_tools', 'jdk23_tools')
SIDES = ('original', 'jadx', 'jarde')
EXPECTED_RUNTIME_LINES = [
    b'labels:-2147483648:Integer:0:Integer:0',
    b'labels:-1:Integer:0:Integer:0',
    b'labels:0:Integer:0:Integer:0',
    b'labels:2747:Integer:0:Integer:0',
    b'labels:2748:Integer:3294:Integer:0',
    b'labels:2749:Integer:0:Integer:0',
    b'labels:3293:Integer:0:Integer:0',
    b'labels:3294:Integer:0:Integer:2748',
    b'labels:3295:Integer:0:Integer:0',
    b'labels:2147483647:Integer:0:Integer:0',
]


def need(ok, message):
    if not ok:
        raise AssertionError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def verify_pin(pin, label):
    need(isinstance(pin, dict) and isinstance(pin.get('path'), str), f'{label}: malformed path pin')
    p = Path(pin['path'])
    need(p.is_file(), f'{label}: pinned file missing: {p}')
    data = p.read_bytes()
    need(pin.get('bytes') == len(data), f'{label}: byte count differs')
    need(pin.get('sha256') == hashlib.sha256(data).hexdigest(), f'{label}: SHA-256 differs')
    return data


def verify_sha_path(path, expected, label):
    p = Path(path)
    need(p.is_file(), f'{label}: missing: {p}')
    actual = digest(p)
    need(actual == expected, f'{label}: SHA-256 mismatch ({actual})')


def verify_stream(stream, label):
    return verify_pin(stream, label)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('execution', type=Path)
    args = ap.parse_args()
    ex_path = args.execution.resolve()
    ex = json.loads(ex_path.read_text())
    need(ex.get('status') == 'observed_complete_pending_independent_root_review',
         f'unexpected v5 execution status: {ex.get("status")}')
    commands = ex.get('commands')
    need(isinstance(commands, list) and len(commands) == 32, 'expected exactly 32 command rows')
    need([c.get('index') for c in commands] == list(range(32)), 'command indices must be contiguous 0..31')
    by_label = {c.get('label'): c for c in commands}
    need(len(by_label) == 32 and None not in by_label, 'command labels must be unique')

    # Bind the immutable replay inputs and recorded JDK/probe/JADX source inputs.
    need(set(ex['captured_inputs_before']) == set(INPUTS), 'captured input class set changed')
    for name, (size, sha) in INPUTS.items():
        pin = ex['captured_inputs_before'][name]
        need((pin.get('bytes'), pin.get('sha256')) == (size, sha), f'{name}: frozen input metadata differs')
        verify_pin(pin, f'input {name}')
    verify_pin(ex['probe_source'], 'frozen runtime probe source')
    verify_pin(ex['jadx_full_source'], 'frozen JADX source')
    verify_pin(ex['jdk_manifest'], 'frozen JDK manifest')
    verify_pin(ex['new_cli'], 'new CLI')
    verify_pin(ex['old_bafb_cli'], 'old CLI')
    verify_pin(ex['new_build_record'], 'new CLI build record')
    verify_pin(ex['old_bafb_build_record'], 'old CLI build record')
    build_binding = ex['new_build_binding']
    verify_pin({'path': build_binding['cli']['path'], 'bytes': build_binding['cli']['size_bytes'],
                'sha256': build_binding['cli']['sha256']}, 'new build-binding CLI')
    need(build_binding['cli']['sha256'] == ex['new_cli']['sha256'], 'build-binding CLI differs from execution CLI')
    need(build_binding.get('live_source_pin_count') == len(build_binding.get('live_source_pins', {})),
         'build source pin count mismatch')
    old_pins = ex.get('old_bafb_source_pins', {})
    need(isinstance(old_pins, dict) and set(old_pins).issubset(build_binding['live_source_pins']),
         'old baseline source pins are not a subset of the new source manifest')
    source_count = 0
    for path, sha in build_binding['live_source_pins'].items():
        verify_sha_path(path, sha, 'live source pin')
        source_count += 1

    # Every command must be represented exactly once by its captured index, with both streams pinned.
    command_streams = {}
    for c in commands:
        label = c['label']
        need(c.get('exit_code') == 0 and c.get('guard_stop') in (None, ''), f'{label}: command did not exit cleanly')
        need(isinstance(c.get('argv'), list) and c['argv'], f'{label}: missing argv')
        need(isinstance(c.get('cwd'), str) and Path(c['cwd']).is_dir(), f'{label}: invalid cwd')
        streams = c.get('streams', {})
        need(set(streams) == {'stdout', 'stderr'}, f'{label}: expected exactly stdout/stderr records')
        stdout = verify_stream(streams['stdout'], f'{label} stdout')
        stderr = verify_stream(streams['stderr'], f'{label} stderr')
        command_streams[label] = (stdout, stderr)

    # Eight real CLI render records: exact JDK-independent Cartesian rows old/new × mode × class.
    expected_render = {(group, mode, cls) for group in ('old_bafb', 'new_cf12')
                       for mode in MODES for cls in INPUTS}
    seen_render = set()
    render_expected_cli = {'old_bafb': ex['old_bafb_cli'], 'new_cf12': ex['new_cli']}
    for group, rows in ex['fresh_cli_renders'].items():
        need(group in render_expected_cli and isinstance(rows, list) and len(rows) == 4,
             f'unexpected fresh render group/size: {group}')
        for row in rows:
            cls, mode = row['class'], row['mode']
            key = (group, mode, cls)
            need(key in expected_render and key not in seen_render, f'unexpected/duplicate render: {key}')
            seen_render.add(key)
            c = row['command']
            need(c['index'] < len(commands) and commands[c['index']] == c,
                 f'{key}: inline command differs from top-level indexed record')
            argv = c['argv']
            cli = render_expected_cli[group]
            need(Path(argv[0]).resolve() == Path(cli['path']).resolve(), f'{key}: wrong CLI executable')
            need(Path(argv[0]).is_file() and digest(argv[0]) == cli['sha256'], f'{key}: CLI SHA mismatch')
            need(argv[1:2] == ['class-source'], f'{key}: wrong CLI operation')
            input_path = argv[argv.index('--input') + 1]
            need(Path(input_path).resolve() == Path(ex['captured_inputs_before'][cls]['path']).resolve(),
                 f'{key}: wrong frozen class input')
            need(argv[argv.index('--class') + 1] == 'jadx.tests.integration.switches.' + cls,
                 f'{key}: wrong selected binary class')
            pin = row['raw_json']
            need(Path(pin['path']).resolve() == Path(c['streams']['stdout']['path']).resolve(),
                 f'{key}: raw JSON does not point at command stdout')
            need(pin['bytes'] == len(command_streams[c['label']][0]) and
                 pin['sha256'] == hashlib.sha256(command_streams[c['label']][0]).hexdigest(),
                 f'{key}: raw JSON bytes/SHA differs from captured stdout')
            doc = json.loads(command_streams[c['label']][0])
            need(doc.get('class', {}).get('class_bytes', {}).get('digest') ==
                 ('4080d98724a27c977ba6668219e6c010c321aef2f30f88ace3a57070b32095a1'
                  if cls == ROOT_CLASS else '38b78ad84ad58522f6f863102670f46db563c5491374bacbb19ad0eae1ea71b7'),
                 f'{key}: report class identity is not the frozen class')
    need(seen_render == expected_render, 'fresh render rows are not the exact 2×2×2 set')

    profiles = ex.get('profiles')
    expected_profiles = {(jdk, mode) for jdk in JDKS for mode in MODES}
    need(isinstance(profiles, list) and len(profiles) == 4, 'expected exactly four JDK×mode profiles')
    need({(p.get('jdk'), p.get('mode')) for p in profiles} == expected_profiles,
         'profiles are not the exact JDK×mode Cartesian set')
    expected_entries = {
        PACKAGE_PATH + ROOT_CLASS + '.class',
        PACKAGE_PATH + INNER_CLASS + '.class',
    }
    actual_runtime_rows = 0
    compile_count = runtime_count = 0
    for p in profiles:
        jdk, mode = p['jdk'], p['mode']
        key = f'{jdk}/{mode}'
        need(p.get('runtime_streams_equal') is True, f'{key}: runner did not record stream equality')
        tools = p['tool_pins']
        need(set(tools) == {'javac', 'java'}, f'{key}: expected javac/java pins')
        for tool, pin in tools.items():
            verify_pin(pin, f'{key} {tool}')
        need(p.get('original_class_entries') == sorted(expected_entries), f'{key}: original entry set differs')
        for side in ('jarde', 'jadx'):
            need(p.get(f'{side}_class_set_equal_original') is True, f'{key}: {side} class set mismatch')

        # Verify all exact class/source copies recorded for this profile.
        original_pins = p['original_class_pins']
        need(set(original_pins) == set(INPUTS), f'{key}: original pins do not name exactly two classes')
        for cls, inp in INPUTS.items():
            op = original_pins[cls]
            need((op['bytes'], op['sha256']) == inp, f'{key}/{cls}: original class differs from frozen input')
            verify_pin(op, f'{key} original class {cls}')
        for field, count in [('probe_class_pins', 1), ('jarde_source_pins', 2),
                             ('jarde_compiled_class_pins', 2), ('jadx_compiled_class_pins', 2)]:
            pins = p[field]
            need(isinstance(pins, list) and len(pins) == count, f'{key}: {field} expected {count} pins')
            for i, pin in enumerate(pins):
                verify_pin(pin, f'{key} {field}[{i}]')
        for field in ('probe_source_copy', 'jadx_source_copy', 'jadx_source_input'):
            verify_pin(p[field], f'{key} {field}')
        for field in ('jarde_compiled_class_pins', 'jadx_compiled_class_pins'):
            names = {Path(pin['path']).as_posix().split('/classes/', 1)[-1] for pin in p[field]}
            need(names == expected_entries, f'{key}: {field} does not contain the exact two expected class entries')
        need(p['probe_source_copy']['sha256'] == ex['probe_source']['sha256'], f'{key}: probe source copy differs')
        need(p['jadx_source_copy']['sha256'] == p['jadx_source_input']['sha256'], f'{key}: JADX source copy differs')

        # Exactly three successful compile commands: JADX, Jarde, and runtime probe.
        compile_refs = p['compile_commands']
        need(isinstance(compile_refs, list) and len(compile_refs) == 3, f'{key}: expected three compile commands')
        compile_sides = set()
        for ref in compile_refs:
            c = commands[ref['index']]
            need(c == ref and c['exit_code'] == 0, f'{key}: compile ref mismatch or failure')
            label = c['label']
            need(label.startswith(f'{jdk}-{mode}-') and label.endswith('-compile'), f'{key}: compile label mismatch')
            argv = c['argv']
            need(Path(argv[0]).resolve() == Path(tools['javac']['path']).resolve(), f'{key}: compile used wrong javac')
            side = next((s for s in ('jadx', 'jarde', 'probe') if f'-{s}-compile' in label), None)
            need(side is not None and side not in compile_sides, f'{key}: duplicate/unknown compile side')
            compile_sides.add(side)
            out_index = argv.index('-d')
            outdir = argv[out_index + 1]
            expected_out = f'{ex_path.parent}/{jdk}/{mode}/runtime-probe' if side == 'probe' else f'{ex_path.parent}/{jdk}/{mode}/{side}/classes'
            need(Path(outdir).resolve() == Path(expected_out).resolve(), f'{key}/{side}: wrong output directory')
            sources = {Path(x).resolve() for x in argv[out_index + 2:]}
            if side == 'probe':
                expected_sources = {Path(p['probe_source_copy']['path']).resolve()}
            elif side == 'jadx':
                expected_sources = {Path(p['jadx_source_copy']['path']).resolve()}
            else:
                expected_sources = {Path(pin['path']).resolve() for pin in p['jarde_source_pins']}
            need(sources == expected_sources, f'{key}/{side}: compile sources differ from frozen source pins')
            compile_count += 1
        need(compile_sides == {'jadx', 'jarde', 'probe'}, f'{key}: compile sides incomplete')

        # Exact original/JADX/Jarde runtime commands and probe/classpath arguments.
        runtime_refs = p['runtime_commands']
        need(isinstance(runtime_refs, list) and len(runtime_refs) == 3, f'{key}: expected three runtime commands')
        runtime_sides = set()
        runtime_outputs = {}
        for ref in runtime_refs:
            c = commands[ref['index']]
            need(c == ref and c['exit_code'] == 0, f'{key}: runtime ref mismatch or failure')
            label = c['label']
            side = next((s for s in SIDES if label.endswith(f'-{s}-runtime')), None)
            need(side is not None and side not in runtime_sides, f'{key}: duplicate/unknown runtime side')
            runtime_sides.add(side)
            argv = c['argv']
            need(Path(argv[0]).resolve() == Path(tools['java']['path']).resolve(), f'{key}/{side}: wrong java')
            need('-Xverify:all' in argv, f'{key}/{side}: verification flag missing')
            cp = argv[argv.index('-cp') + 1].split(':')
            expected_cp = [f'{ex_path.parent}/{jdk}/{mode}/runtime-probe',
                           f'{ex_path.parent}/{jdk}/{mode}/{side}/classes']
            need(cp == expected_cp, f'{key}/{side}: wrong classpath {cp}')
            need(argv[argv.index('Cf12RuntimeProbe') + 1:] == [BINARY, 'TestSwitchLabels'],
                 f'{key}/{side}: wrong probe target arguments')
            stdout, stderr = command_streams[label]
            lines = [line for line in stdout.splitlines() if line.strip()]
            need(lines == EXPECTED_RUNTIME_LINES, f'{key}/{side}: runtime rows do not equal the exact ten input observations')
            need(not stderr.strip(), f'{key}/{side}: runtime stderr is not empty')
            runtime_outputs[side] = (stdout, stderr)
            runtime_count += 1
        need(runtime_sides == set(SIDES), f'{key}: runtime sides incomplete')
        need(runtime_outputs['original'] == runtime_outputs['jadx'] == runtime_outputs['jarde'],
             f'{key}: actual ten-row runtime output differs across original/JADX/Jarde')
        actual_runtime_rows += 10 * len(SIDES)

    need(compile_count == 12 and runtime_count == 12, 'expected 12 compile and 12 runtime commands')
    print(json.dumps({'status': 'v5_captured_runtime_manifest_verified', 'commands': len(commands),
                      'fresh_cli_renders': len(seen_render), 'profiles': len(profiles),
                      'compile_commands': compile_count, 'runtime_commands': runtime_count,
                      'runtime_observation_rows': actual_runtime_rows,
                      'source_pins_verified': source_count,
                      'runtime_rows_per_profile_side': 10,
                      'note': 'read-only verification of captured v5 records; no tools launched'}, indent=2))

if __name__ == '__main__':
    main()
