#!/usr/bin/env python3
"""Verify captured runner-v2 full-class replay records; this script runs no tools."""
import argparse
import hashlib
import json
from pathlib import Path


def require(ok, msg):
    if not ok:
        raise AssertionError(msg)


def rows_for(value):
    if isinstance(value, list):
        out = []
        for item in value:
            out.extend(rows_for(item))
        return out
    if isinstance(value, dict):
        if 'argv' in value and 'exit_code' in value:
            return [value]
        out = []
        for item in value.values():
            if isinstance(item, (dict, list)):
                out.extend(rows_for(item))
        return out
    return []


def command_from_ref(execution, ref):
    commands = execution.get('commands', [])
    if isinstance(ref, dict):
        idx = ref.get('index', ref.get('command_index'))
        if idx is not None:
            ref = idx
        elif 'argv' in ref and 'exit_code' in ref:
            candidates = [c for c in commands if c.get('argv') == ref.get('argv')
                          and c.get('streams') == ref.get('streams')]
            require(len(candidates) == 1, 'inline profile command must uniquely match execution.commands')
            return candidates[0]
        else:
            ref = ref.get('label')
    if isinstance(ref, int):
        matches = [c for i, c in enumerate(commands)
                   if c.get('index') == ref or (c.get('index') is None and i == ref)]
    else:
        matches = [c for c in commands if c.get('label') == ref]
    require(len(matches) == 1, f'profile command reference {ref!r} is not unique')
    return matches[0]


def command_rows(execution, refs):
    out = []

    def visit(value, side_hint=None):
        if isinstance(value, list):
            for item in value:
                visit(item, side_hint)
            return
        if isinstance(value, dict):
            if not any(k in value for k in ('argv', 'index', 'command_index', 'label')):
                for key, item in value.items():
                    visit(item, key if key in {'original', 'jadx', 'jarde'} else side_hint)
                return
            row = command_from_ref(execution, value)
        else:
            row = command_from_ref(execution, value)
        if side_hint and not row.get('side'):
            row = dict(row)
            row['_side_hint'] = side_hint
        out.append(row)

    visit(refs)
    return out


def class_entries(profile, side):
    direct = profile.get(f'{side}_class_entries')
    if direct is not None:
        return direct
    class_sets = profile.get('class_sets', {})
    value = class_sets.get(side) if isinstance(class_sets, dict) else None
    if isinstance(value, dict):
        return value.get('entries')
    return value


def entry_names(entries):
    names = []
    for entry in entries:
        if isinstance(entry, str):
            names.append(entry)
        elif isinstance(entry, dict):
            name = entry.get('relative_path', entry.get('path', entry.get('name')))
            require(isinstance(name, str), 'class entry record lacks a relative path/name')
            names.append(name)
        else:
            raise AssertionError('class entry must be a path or pinned entry record')
    return names


def side_for(row):
    side = row.get('side', row.get('_side_hint'))
    if side in {'original', 'jadx', 'jarde'}:
        return side
    label = str(row.get('label', '')).lower()
    for candidate in ('original', 'jadx', 'jarde'):
        if candidate in label:
            return candidate
    return None


def verify_stream(stream, label):
    path = Path(stream['path'])
    data = path.read_bytes()
    require(stream.get('bytes') == len(data), f'{label}: stream byte count mismatch')
    require(stream.get('sha256') == hashlib.sha256(data).hexdigest(), f'{label}: stream SHA mismatch')
    return data


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('execution', type=Path, help='captured runner-v2 execution.json')
    args = ap.parse_args()
    execution = json.loads(args.execution.read_text())
    profiles = execution.get('profiles')
    require(isinstance(profiles, list), 'execution requires profiles[]')
    require(len(profiles) == 4, f'expected two JDK legs × two profiles; got {len(profiles)}')
    require({p.get('mode', p.get('profile')) for p in profiles} == {'default', 'all'},
            'profiles must contain default and all')
    require(len({p.get('jdk') for p in profiles}) == 2, 'profiles must contain two JDK legs')
    seen = set()
    runtime_rows_total = 0
    for profile in profiles:
        mode = profile.get('mode', profile.get('profile'))
        key = (profile.get('jdk'), mode)
        require(key not in seen, f'duplicate profile {key}')
        seen.add(key)

        compile_refs = profile.get('compile_commands')
        require(compile_refs, f'{key}: missing compile_commands')
        compile_rows = command_rows(execution, compile_refs)
        for row in compile_rows:
            require(row.get('exit_code') == 0 and row.get('guard_stop') in (None, ''),
                    f'{key}: a recorded full-source compile command did not exit zero')

        original = class_entries(profile, 'original')
        candidate = class_entries(profile, 'jarde')
        require(isinstance(original, list) and original, f'{key}: no actual original class-entry inventory')
        require(isinstance(candidate, list) and candidate, f'{key}: no actual Jarde class-entry inventory')
        original_names, candidate_names = entry_names(original), entry_names(candidate)
        require(len(original_names) == len(set(original_names)) and len(candidate_names) == len(set(candidate_names)),
                f'{key}: duplicate full-class-set entry')
        require(set(candidate_names) == set(original_names), f'{key}: Jarde compiled class set differs from original')

        refs = profile.get('runtime_commands')
        require(refs, f'{key}: missing runtime_commands')
        runtime_cmds = command_rows(execution, refs)
        side_rows = {}
        for row in runtime_cmds:
            side = side_for(row)
            require(side is not None and side not in side_rows,
                    f'{key}: runtime side missing or duplicated for command {row.get("label")}')
            side_rows[side] = row
        require(set(side_rows) == {'original', 'jadx', 'jarde'},
                f'{key}: expected original/JADX/Jarde full-class runtimes, got {sorted(side_rows)}')
        # Do not compare streams until every process has a successful status and verifier flag.
        for side, row in side_rows.items():
            require(row.get('exit_code') == 0 and row.get('guard_stop') in (None, ''),
                    f'{key}/{side}: runtime did not exit zero cleanly')
            argv = row.get('argv', [])
            require('-Xverify:all' in argv, f'{key}/{side}: JVM invocation omitted -Xverify:all')
            require('Cf12RuntimeProbe' in argv, f'{key}/{side}: wrong probe entry point')
        streams = {}
        for side, row in side_rows.items():
            output = verify_stream(row.get('streams', {}).get('stdout', {}), f'{key}/{side} stdout')
            error = verify_stream(row.get('streams', {}).get('stderr', {}), f'{key}/{side} stderr')
            nonempty = [line for line in output.splitlines() if line.strip()]
            require(len(nonempty) == 10,
                    f'{key}/{side}: expected ten actual runtime rows, got {len(nonempty)}')
            streams[side] = (output, error)
            runtime_rows_total += len(nonempty)
        require(streams['jarde'] == streams['original'], f'{key}: Jarde runtime differs from original')
        require(streams['jadx'] == streams['original'], f'{key}: JADX control runtime differs from original')

    print(json.dumps({
        'status': 'dual_jdk_full_class_replay_verified',
        'profiles': len(profiles),
        'jdk_legs': sorted({p['jdk'] for p in profiles}),
        'modes': ['default', 'all'],
        'runtime_observation_rows_total': runtime_rows_total,
        'note': 'result covers only the captured TestSwitchLabels replay records'
    }, indent=2))

if __name__ == '__main__':
    main()
