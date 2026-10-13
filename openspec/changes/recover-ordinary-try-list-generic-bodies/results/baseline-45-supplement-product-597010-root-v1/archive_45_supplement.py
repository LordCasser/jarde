#!/usr/bin/env python3
"""Create a pinned, reopen-verified archive of the independently audited 45-row baseline supplement."""
import argparse, gzip, hashlib, json, pathlib, stat, tarfile

ROOT = pathlib.Path('/private/tmp')
CAPTURES = {
    'debug': ROOT / 'jarde-ordinary-try-list-45-debug-baseline-root-v1',
    'none': ROOT / 'jarde-ordinary-try-list-45-none-baseline-root-v1',
}
PREVIOUS = {
    'debug': ROOT / 'jarde-ordinary-try-list-debug-baseline-root-v2',
    'none': ROOT / 'jarde-ordinary-try-list-none-baseline-root-v1',
}
ACCEPTANCE = ROOT / 'jarde-ordinary-try-list-45-baseline-acceptance-root-v1.json'
CLEAN = ROOT / 'jarde-ordinary-try-list-45-baseline-final-clean-root-v1'
PREP = ROOT / 'jarde-ordinary-try-list-mvp-baseline-prep-luna-v4'
AUDIT = ROOT / 'jarde-ordinary-try-list-mvp-baseline-audit-luna-v4'
PREVIOUS_RECEIPT_SHA = {
    'debug': 'f9b25f4f93c98c795618590ec75041dd827bd058f344e1c1e8a16dfc92b271e3',
    'none': 'b94ca14e7940a4eb4a964172337393e496ed061195e26af2a8eceb79c53d258b',
}


def digest(data): return hashlib.sha256(data).hexdigest()
def read_json(path): return json.loads(pathlib.Path(path).read_text(encoding='utf-8'))
def pin(path):
    p = pathlib.Path(path).resolve(); data = p.read_bytes()
    return {'path': str(p), 'bytes': len(data), 'sha256': digest(data)}
def verify_pin(path, row):
    actual = pin(path)
    if actual['bytes'] != row.get('bytes') or actual['sha256'] != row.get('sha256'):
        raise ValueError(f'pin mismatch: {path}')
    return actual


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True, type=pathlib.Path, help='new .tar.gz output; existing paths are refused')
    args = parser.parse_args()
    out = args.out.expanduser().resolve()
    sidecar = pathlib.Path(str(out) + '.manifest.json')
    if out.exists() or sidecar.exists(): raise SystemExit(f'refuse overwrite: {out} or {sidecar}')

    acceptance = read_json(ACCEPTANCE)
    if acceptance.get('schema') != 'jarde-ordinary-try-list-mvp-baseline-independent-audit-v4' or acceptance.get('status') != 'audited' or acceptance.get('errors') != []:
        raise ValueError('root v4 acceptance is not audited/error-free')
    accepted = {x.get('mode'): x for x in acceptance.get('captures', [])}
    if set(accepted) != {'debug', 'none'}: raise ValueError('acceptance does not bind exactly debug and none')
    members = {}
    raw_bindings = {'current': {}, 'previous_original_oracles': {}, 'final_clean': []}

    def add(src, member, role):
        src = pathlib.Path(src).resolve(); member = pathlib.PurePosixPath(member).as_posix()
        if member.startswith('/') or '..' in pathlib.PurePosixPath(member).parts: raise ValueError(f'unsafe member: {member}')
        if src.is_symlink() or not src.is_file(): raise ValueError(f'missing or symlink source: {src}')
        data = src.read_bytes(); row = {'member': member, 'bytes': len(data), 'sha256': digest(data),
            'original_path': str(src), 'role': role, 'mode': stat.S_IMODE(src.stat().st_mode)}
        if member in members:
            if members[member]['sha256'] != row['sha256']: raise ValueError(f'duplicate member differs: {member}')
            return members[member]
        members[member] = {'data': data, **row}
        return row

    def add_tree(root, prefix):
        root = pathlib.Path(root).resolve()
        if not root.is_dir(): raise ValueError(f'missing capture: {root}')
        for path in sorted(root.rglob('*')):
            if path.is_symlink(): raise ValueError(f'symlink in capture: {path}')
            if not path.is_file(): continue
            rel = path.relative_to(root)
            if 'target' in rel.parts or path.stat().st_size > 32 * 1024 * 1024:
                raise ValueError(f'forbidden build/tool payload in capture: {path}')
            if path.name in {'java', 'javac', 'javap', 'jadx', 'candidate-cli'} or path.name.endswith('-all.jar'):
                raise ValueError(f'tool executable/binary is excluded: {path}')
            add(path, f'{prefix}/{rel.as_posix()}', f'{prefix} captured evidence/input/output')

    def receipt_raws(mode, capture_root, receipt, prefix):
        rows = []
        for cmd in receipt.get('commands', []):
            for stream, ref in cmd.get('streams', {}).items():
                src = pathlib.Path(ref.get('path', '')).resolve()
                try: rel = src.relative_to(capture_root.resolve()).as_posix()
                except ValueError: raise ValueError(f'{mode} raw outside capture: {src}')
                data = src.read_bytes()
                if len(data) != ref.get('bytes') or digest(data) != ref.get('sha256'):
                    raise ValueError(f'{mode} command raw pin mismatch: {src}')
                member = f'{prefix}/{rel}'
                if member not in members or members[member]['bytes'] != len(data) or members[member]['sha256'] != digest(data):
                    raise ValueError(f'{mode} raw is not packaged exactly: {member}')
                rows.append({'command_index': cmd.get('index'), 'label': cmd.get('label'), 'stream': stream,
                             'member': member, 'bytes': len(data), 'sha256': digest(data), 'receipt_path': str(capture_root / 'execution.json')})
        if len(rows) != 54: raise ValueError(f'{mode}: expected 54 receipt-bound raw streams, got {len(rows)}')
        return rows

    # Current full capture directories retain source/input jars, class files, renders, reports, and all command raws.
    for mode, root in CAPTURES.items():
        receipt_path = root / 'execution.json'; receipt_pin = pin(receipt_path); receipt = read_json(receipt_path)
        accepted_row = accepted[mode]
        if accepted_row.get('status') != 'observed' or accepted_row.get('mode') != mode:
            raise ValueError(f'{mode}: acceptance row status mismatch')
        if accepted_row.get('receipt') != receipt_pin:
            raise ValueError(f'{mode}: acceptance receipt pin mismatch')
        if receipt.get('schema') != 'cf11-target-full-source-capture-v2' or receipt.get('status') != 'observed' or receipt.get('actual_command_count') != 27 or len(receipt.get('commands', [])) != 27:
            raise ValueError(f'{mode}: capture receipt schema/status/count mismatch')
        if receipt.get('source_tool_pins_before') != receipt.get('source_tool_pins_after') or receipt.get('source_tool_inputs_unchanged') is not True:
            raise ValueError(f'{mode}: source/tool pins changed during capture')
        if accepted_row.get('command_count') != 27 or accepted_row.get('raw_stream_count') != 54:
            raise ValueError(f'{mode}: accepted command/raw count mismatch')
        if any(row.get('output_row_count') != 45 or row.get('old_17_rows_match') is not True or row.get('previous_44_rows_match') is not True or row.get('handled_mutable_output_row_match') is not True for row in accepted_row.get('original_rows', [])) or len(accepted_row.get('original_rows', [])) != 2:
            raise ValueError(f'{mode}: accepted 45-row original outcomes mismatch')
        providers = accepted_row.get('provider_compile_runtime', [])
        jadx = [r for r in providers if '-jadx-' in r.get('label', '')]
        jarde = [r for r in providers if '-jarde-' in r.get('label', '')]
        if len(jadx) != 4 or any(r.get('compile_exit') != 1 or r.get('runtime') != 'no-runtime' for r in jadx):
            raise ValueError(f'{mode}: expected four JADX compile-failed/no-runtime results')
        if len(jarde) != 4 or any(r.get('compile_exit') != 0 or r.get('run_exit') != 0 or r.get('runtime') != 'matches-original' for r in jarde):
            raise ValueError(f'{mode}: expected four successful Jarde runtime comparisons')
        add_tree(root, f'captures/{mode}')
        raw_bindings['current'][mode] = receipt_raws(mode, root, receipt, f'captures/{mode}')

    add(ACCEPTANCE, 'acceptance/root-v4-acceptance.json', 'independent root v4 audit acceptance')
    clean_receipt_path = CLEAN / 'execution.json'; clean_receipt = read_json(clean_receipt_path)
    clean_accept = acceptance.get('final_clean', {})
    if clean_accept.get('receipt', {}).get('path') != str(clean_receipt_path) or clean_accept.get('receipt', {}).get('sha256') != pin(clean_receipt_path)['sha256'] or clean_accept.get('status') != 'passed':
        raise ValueError('acceptance does not bind the actual passed final-clean receipt')
    if clean_receipt.get('status') != 'passed' or clean_receipt.get('target_absent_before') is not True or clean_receipt.get('target_absent_after') is not True:
        raise ValueError('final clean evidence is not passed/target-absent')
    add(clean_receipt_path, 'final-clean/execution.json', 'actual final-clean receipt')
    for stream in ('stdout', 'stderr'):
        ref = clean_receipt['command']['streams'][stream]; row = add(ref['path'], f'final-clean/{stream}.raw', 'final-clean raw stream')
        if row['bytes'] != ref.get('bytes') or row['sha256'] != ref.get('sha256'): raise ValueError(f'final-clean {stream} receipt binding mismatch')
        raw_bindings['final_clean'].append({'stream': stream, 'member': row['member'], 'receipt_path': str(clean_receipt_path), 'bytes': row['bytes'], 'sha256': row['sha256']})

    # Preserve the old 44-row original oracle receipts and only their two per-JDK original stdout/stderr streams.
    for mode, root in PREVIOUS.items():
        receipt_path = root / 'execution.json'; rp = pin(receipt_path)
        if rp['sha256'] != PREVIOUS_RECEIPT_SHA[mode]: raise ValueError(f'{mode}: old 44-row receipt pin changed')
        receipt = read_json(receipt_path)
        if receipt.get('schema') != 'cf11-target-full-source-capture-v2' or receipt.get('status') != 'observed' or receipt.get('actual_command_count') != 27:
            raise ValueError(f'{mode}: previous capture receipt mismatch')
        add(receipt_path, f'previous-44/{mode}/execution.json', 'accepted 44-row prefix receipt')
        wanted = {f'javac8-original-run', f'javac23-original-run'}
        commands = {c.get('label'): c for c in receipt.get('commands', []) if c.get('label') in wanted}
        if set(commands) != wanted: raise ValueError(f'{mode}: original run commands missing from prior receipt')
        for label, cmd in commands.items():
            stdout_ref = cmd['streams']['stdout']; stdout = pathlib.Path(stdout_ref['path']).read_bytes()
            if len(stdout) != stdout_ref.get('bytes') or digest(stdout) != stdout_ref.get('sha256') or len(stdout.splitlines()) != 44:
                raise ValueError(f'{mode}:{label}: old stdout is not pinned 44-row output')
            current_jdk = label.split('-', 1)[0]
            new_cmd = next(c for c in read_json(CAPTURES[mode] / 'execution.json')['commands'] if c.get('label') == label)
            new_stdout = pathlib.Path(new_cmd['streams']['stdout']['path']).read_bytes()
            if not new_stdout.startswith(stdout): raise ValueError(f'{mode}:{label}: current stdout lacks exact 44-row prefix')
            for stream in ('stdout', 'stderr'):
                ref = cmd['streams'][stream]
                member = f'previous-44/{mode}/raw/{current_jdk}.{stream}.raw'
                row = add(ref['path'], member, 'prior original-run raw stream for 44-row prefix verification')
                if row['bytes'] != ref.get('bytes') or row['sha256'] != ref.get('sha256'): raise ValueError('old raw receipt binding mismatch')
                raw_bindings['previous_original_oracles'].setdefault(mode, []).append({'jdk': current_jdk, 'stream': stream, 'member': member, 'receipt_path': str(receipt_path), 'bytes': row['bytes'], 'sha256': row['sha256']})

    # Include only relevant source/prep/audit files; pinned tool binaries remain path/hash references.
    def pinned_row(ledger, path):
        resolved = str(pathlib.Path(path).resolve())
        rows = [r for r in ledger.get('sources', []) if r.get('path') == resolved]
        if len(rows) != 1: raise ValueError(f'exact source pin missing or ambiguous: {resolved}')
        return verify_pin(resolved, rows[0])
    prep_ledger = read_json(PREP / 'pins.json')
    audit_ledger = read_json(AUDIT / 'pins.json')
    prep_files = ['README.md', 'candidate.diff', 'collect_controls.py', 'run-current.sh', 'static-check.json',
                  'inputs/Runner.java', 'inputs/UpstreamNestedLoops.java', 'inputs/manifest.json']
    for name in prep_files:
        pinned_row(prep_ledger, PREP / name)
        add(PREP / name, f'prep-v4/{name}', 'v4 baseline input/collector preparation')
    audit_files = ['README.md', 'audit.py', 'diff-v3.patch', 'static-check.json']
    for name in audit_files:
        pinned_row(audit_ledger, AUDIT / name)
        add(AUDIT / name, f'auditor-v4/{name}', 'independent auditor source, diff, and static record')
    add(AUDIT / 'pins.json', 'auditor-v4/pins.json', 'v4 auditor source/tool pin ledger')
    add(PREP / 'pins.json', 'prep-v4/pins.json', 'v4 prep source pin ledger')

    # Copy small external source dependencies referenced by capture pins; retain binary/tool references only as JSON paths and hashes.
    tool_refs = {}
    for mode, root in CAPTURES.items():
        receipt = read_json(root / 'execution.json')
        for ref in receipt.get('source_tool_pins_before', []):
            tool_refs[ref['path']] = ref
    external_rows = []
    for src, ref in sorted(tool_refs.items()):
        row = dict(ref); row['included'] = False; row['archive_member'] = None
        path = pathlib.Path(src)
        if path.suffix == '.py' or path.name == 'manifest.json':
            if path.exists():
                member = add(path, f'pinned-tools/{path.stem}-{ref["sha256"][:12]}{path.suffix}', 'small pinned verifier/guard/scanner/JDK-manifest source')
                if member['bytes'] != ref.get('bytes') or member['sha256'] != ref.get('sha256'):
                    raise ValueError(f'external tool pin mismatch: {path}')
                row['included'] = True; row['archive_member'] = member['member']
        external_rows.append(row)
    external_doc = json.dumps({'schema':'jarde-baseline-external-tool-pins-v1','references':external_rows}, sort_keys=True, indent=2).encode() + b'\n'
    external_member = 'tool-references/external-tool-pins.json'
    members[external_member] = {'data':external_doc,'member':external_member,'bytes':len(external_doc),'sha256':digest(external_doc),'original_path':'generated from capture receipt source_tool_pins_before','role':'external tool path/hash references; large CLI/JDK/JADX binaries intentionally not packaged','mode':0o644}

    readme = ('# Ordinary try/List baseline 45-row supplement\n\n'
      'Root v4 acceptance is `audited` with no errors. Both debug and none captures contain 27 commands and 54 receipt-bound raw streams; each original JDK run has 45 rows and an exact same-mode 44-row prefix. The added row is `try-catch-handler-mutable-output`.\n\n'
      'Jarde has four successful compile/runtime comparison legs per mode, matching the original. JADX has four compile-failed/no-runtime legs per mode. These are baseline observations; this archive does not claim an implementation change. Final exact-clean receipt is included and records target absent before and after.\n\n'
      'The complete captures retain source/input jars, class files, generated sources, reports, render output, receipts, and raw streams. Large CLI/JDK/JADX executables and Cargo target output are excluded; their pinned paths, sizes, and hashes remain in `tool-references/external-tool-pins.json`. The manifest records every member byte length/SHA-256 and receipt binding.\n')
    members['README.md']={'data':readme.encode(),'member':'README.md','bytes':len(readme.encode()),'sha256':digest(readme.encode()),'original_path':'generated by archive script','role':'scope and interpretation','mode':0o644}

    # Manifest carries original paths and direct receipt-to-raw associations for independent reopening.
    manifest_doc = {'schema':'jarde-ordinary-try-list-45-baseline-archive-v1','acceptance_status':'audited','interpretation':'45-row debug/none full-class baseline supplement only; no implementation claim','raw_receipt_bindings':raw_bindings,
      'members':[{'member':v['member'],'bytes':v['bytes'],'sha256':v['sha256'],'original_path':v['original_path'],'role':v['role']} for _,v in sorted(members.items())]}
    manifest_bytes = json.dumps(manifest_doc, ensure_ascii=False, sort_keys=True, indent=2).encode()+b'\n'
    members['MANIFEST.json']={'data':manifest_bytes,'member':'MANIFEST.json','bytes':len(manifest_bytes),'sha256':digest(manifest_bytes),'original_path':'generated by archive script','role':'complete per-member archive manifest with raw receipt bindings','mode':0o644}

    out.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(out, 'w:gz', format=tarfile.PAX_FORMAT) as tar:
        for name, row in sorted(members.items()):
            info=tarfile.TarInfo(name); info.size=len(row['data']); info.mode=row['mode']; info.mtime=0
            import io; tar.addfile(info, io.BytesIO(row['data']))
    # Reopen every member and compare against the exact in-memory manifest rows.
    with tarfile.open(out, 'r:gz') as tar:
        seen=set()
        for info in tar.getmembers():
            if not info.isfile() or info.name in seen: raise ValueError(f'invalid archive member: {info.name}')
            seen.add(info.name); data=tar.extractfile(info).read(); row=members.get(info.name)
            if row is None or len(data)!=row['bytes'] or digest(data)!=row['sha256']:
                raise ValueError(f'reopened archive member mismatch: {info.name}')
        if seen != set(members): raise ValueError('reopened member set differs from manifest')
        embedded=json.loads(tar.extractfile('MANIFEST.json').read())
        for row in embedded['members']:
            data=tar.extractfile(row['member']).read()
            if len(data)!=row['bytes'] or digest(data)!=row['sha256']: raise ValueError(f'manifest verification failed: {row["member"]}')
    archive_pin=pin(out)
    sidecar_doc={'schema':'jarde-ordinary-try-list-45-baseline-archive-sidecar-v1','archive':archive_pin,'members':manifest_doc['members']+[{'member':'MANIFEST.json','bytes':len(manifest_bytes),'sha256':digest(manifest_bytes),'original_path':'generated by archive script','role':'embedded manifest'}]}
    sidecar.write_text(json.dumps(sidecar_doc,ensure_ascii=False,sort_keys=True,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'archive':str(out),'bytes':archive_pin['bytes'],'sha256':archive_pin['sha256'],'members':len(members),'raw_streams_current':108,'acceptance':'audited'},indent=2))

if __name__ == '__main__': main()
