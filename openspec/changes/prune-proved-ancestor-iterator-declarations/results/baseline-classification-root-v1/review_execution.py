#!/usr/bin/env python3
"""Independent integrity and outcome classifier for the ancestor iterator collector.

This is read-only. It verifies receipt pins, command streams, source/class closures,
raw API outputs, and exact runtime oracles. It reports product coverage separately;
receipt status `captured-pending-review` is never treated as acceptance.
"""
from __future__ import annotations
import argparse, base64, hashlib, json, re, sys, zipfile
from pathlib import Path

SCHEMA = 'cf11-iterator-ancestor-controls-matrix-v1'
BASE_SHA = '1fbc03d82b6255275d31e0ce75624ef896a73491a50e09670927062e4318a953'
KNOWN_SOURCE_DRIVER_PAIRS = {
    ('cdd3ae0b259156d1e22de9484e4a151f80d0acca373d19352cddf40518a6f0ac','c404b4444940c65ac573d5f667e14f4c60660a66ec633df0d8bf8b7d5d7a51c9'),
    ('77c6d999dec1ca32d69b4cac87a1f2a6e5d7ef2b5139cf155d016a942dff5aa0','c404b4444940c65ac573d5f667e14f4c60660a66ec633df0d8bf8b7d5d7a51c9'),
    ('3e03abfb85acc37bee89142148985ed253b2bf7b0efc01a2e6f78c025523c26a','c404b4444940c65ac573d5f667e14f4c60660a66ec633df0d8bf8b7d5d7a51c9'),
    ('3e03abfb85acc37bee89142148985ed253b2bf7b0efc01a2e6f78c025523c26a','2838f4593fac8da7b4ad33f47730a5e7e7f5d192a45f314bc2a0ab66eb0fb2fa'),
}
KNOWN_COLLECTOR_PARSE_ERROR = "malformed driver case row: 'CASE name=protected-hasNext-throws"
RUNTIME_SHA = '73d1bedf469b7bb87d6fc860b12660fd6f4834d022f72c88b6c093e98ce312ca'
INTERNAL = 'cf11/IteratorAncestorCases'
PROFILES = ('javac8-debug', 'javac8-no-debug', 'javac23-debug', 'javac23-no-debug')
EVIDENCE = ('default', 'all', 'source_map')
VMS = ('javac8', 'javac23')
METHODS = {
    ('<init>', '()V'), ('ancestor', '(Ljava/util/List;)I'),
    ('sameList', '(Ljava/util/List;)I'), ('protectedPositive', '(Ljava/util/List;)I'),
    ('protectedBeforeRead', '(Ljava/util/List;)I'), ('protectedAfterRead', '(Ljava/util/List;)I'),
    ('protectedAfterWrite', '(Ljava/util/List;Ljava/util/List;)I'),
    ('protectedCatchReadWrite', '(Ljava/util/List;Ljava/util/List;)I'),
    ('reusedSlots', '(Ljava/util/List;Ljava/util/List;)I'),
    ('siblingNames', '(Ljava/util/List;Ljava/util/List;Z)I'),
}
CASES = (
    ('ancestor-empty','ancestor'), ('ancestor-normal','ancestor'),
    ('ancestor-null-element','ancestor'), ('ancestor-wrong-after-success','ancestor'),
    ('ancestor-null-source','ancestor'), ('same-list-empty','sameList'),
    ('same-list-normal','sameList'), ('same-list-wrong-element','sameList'),
    ('protected-normal','protectedPositive'), ('protected-hasNext-throws','protectedPositive'),
    ('protected-next-throws','protectedPositive'), ('protected-before-read','protectedBeforeRead'),
    ('protected-after-read','protectedAfterRead'), ('protected-after-write','protectedAfterWrite'),
    ('protected-catch-read-write','protectedCatchReadWrite'), ('reused-slots-both','reusedSlots'),
    ('reused-slots-empty-left','reusedSlots'), ('sibling-name-left','siblingNames'),
    ('sibling-name-right','siblingNames'),
)
LINE = re.compile(r'^CASE name=([A-Za-z0-9-]+) method=([A-Za-z][A-Za-z0-9]*) exception=([^ ]+) returned=([^ ]+) observed=(-?\d+) trace=([^ ]*)$')
FOREACH = re.compile(r'\bfor\s*\([^;\n]+:\s*[^)\n]+\)')
BLANK_DECL = re.compile(r'^\s*[A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*(?:\s*<[^;{}]+>)?(?:\[\])?\s+[A-Za-z_$][\w$]*\s*;\s*$')

def sha_bytes(data: bytes) -> str: return hashlib.sha256(data).hexdigest()
def sha_file(path: Path) -> str:
    h=hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda:f.read(1<<20),b''): h.update(b)
    return h.hexdigest()
def fail(errors, message): errors.append(message)
def checked_pin(pin, errors, label):
    if not isinstance(pin,dict) or not {'path','bytes','sha256'} <= set(pin):
        fail(errors,f'{label}: malformed pin'); return None
    p=Path(pin['path'])
    try:
        data=p.read_bytes()
    except Exception as e:
        fail(errors,f'{label}: cannot read {p}: {e}'); return None
    actual={'path':str(p.resolve()),'bytes':len(data),'sha256':sha_bytes(data)}
    if actual != pin: fail(errors,f'{label}: pin mismatch: {p}')
    return data
def checked_stream(row, channel, errors):
    return checked_pin(row.get('streams',{}).get(channel),errors,f'command {row.get("index")} {channel}')
def parse_rows(data, label, errors):
    try: lines=data.decode('utf-8','strict').splitlines()
    except Exception as e: fail(errors,f'{label}: stdout is not UTF-8: {e}'); return []
    out=[]
    for line in lines:
        m=LINE.fullmatch(line)
        if not m: fail(errors,f'{label}: malformed CASE line: {line[:100]!r}'); continue
        name,method,ex,ret,observed,trace=m.groups()
        if name not in dict(CASES) or dict(CASES)[name] != method: fail(errors,f'{label}: unexpected case/method {name}/{method}')
        try: base64.b64decode(ret,validate=True); base64.b64decode(trace,validate=True)
        except Exception: fail(errors,f'{label}: invalid base64 field in {name}')
        out.append(line)
    if len(out)!=19 or [LINE.fullmatch(x).group(1) for x in out if LINE.fullmatch(x)] != [x[0] for x in CASES]:
        fail(errors,f'{label}: expected exact ordered 19-case output, got {len(out)} rows')
    return out
def pin_tree(tree, errors, label):
    if not isinstance(tree,dict): fail(errors,f'{label}: missing tree'); return set()
    names=set()
    for name,pin in tree.items():
        names.add(name); checked_pin(pin,errors,f'{label}/{name}')
    return names
def method_key(m): return (m.get('name'),m.get('descriptor'))
def classify_text(text):
    lines=text.splitlines()
    return {'foreach_headers':len(FOREACH.findall(text)),
            'blank_local_declarations':[line.strip() for line in lines if BLANK_DECL.fullmatch(line)],
            'quality':None}
def source_class(report, cell, errors):
    p=checked_pin(cell.get('report'),errors,f'API report {cell.get("profile")}/{cell.get("evidence")}')
    if p is None: return None
    try: doc=json.loads(p.decode('utf-8','strict'))
    except Exception as e: fail(errors,f'API report JSON invalid: {e}'); return None
    if doc.get('execution',{}).get('status')!='complete': fail(errors,'API execution status not complete')
    whole=cell.get('whole_source')
    data=checked_pin(whole,errors,f'API whole source {cell.get("profile")}/{cell.get("evidence")}')
    text=doc.get('text')
    if not isinstance(text,str) or data is None or data != text.encode('utf-8'):
        fail(errors,'API report whole text differs from saved source bytes')
        return doc
    if cell.get('source_ready') is not True: fail(errors,'API source_ready false despite whole source')
    actual_methods=[]; derived=[]
    for m in doc.get('methods',[]):
        ident=m.get('item',{}).get('identity',{})
        try: key=(bytes(ident['name']).decode('utf-8'),bytes(ident['descriptor']).decode('utf-8'))
        except Exception: fail(errors,'API method identity malformed'); continue
        actual_methods.append(key)
        outcome=m.get('outcome',{}).get('report',{})
        mt=outcome.get('text')
        quality=outcome.get('quality')
        sm=outcome.get('source_map')
        if not isinstance(mt,str) or not isinstance(quality,str): fail(errors,f'API raw method text/quality missing for {key}')
        if not isinstance(sm,dict) or not isinstance(sm.get('segments'),list): fail(errors,f'API source map missing/malformed for {key}')
        if isinstance(mt,str) and isinstance(sm,dict):
            profile_facts=(report.get('source_profiles') or [])
            profile_row=next((x for x in profile_facts if x.get('profile')==cell.get('profile')),None)
            class_facts=((profile_row or {}).get('class_facts') or {}).get(INTERNAL+'.class',{})
            physical={method_key(x):set((x.get('code') or {}).get('bcis',[])) for x in class_facts.get('methods',[])}
            class_digest=doc.get('class',{}).get('class_bytes',{}).get('digest')
            class_length=doc.get('class',{}).get('class_bytes',{}).get('length')
            physical_pin=((profile_row or {}).get('class_tree') or {}).get(INTERNAL+'.class',{})
            if class_length!=physical_pin.get('bytes'): fail(errors,f'API report class byte length differs from physical class for {key}')
            def inspect_origin(value):
                if isinstance(value,dict):
                    if isinstance(value.get('bci'),int) and isinstance(value.get('method'),dict):
                        ident=value['method']
                        try: origin_key=(bytes(ident['name']).decode('utf-8'),bytes(ident['descriptor']).decode('utf-8'))
                        except Exception: origin_key=None
                        if origin_key not in physical or value['bci'] not in physical.get(origin_key,set()):
                            fail(errors,f'API source map references unknown physical BCI for {key}: {origin_key}/{value["bci"]}')
                        owner=value.get('method',{}).get('owner',{})
                        digest=owner.get('class_bytes',{}).get('digest')
                        if class_digest and digest!=class_digest: fail(errors,f'API source map class digest differs for {key}')
                    for child in value.values(): inspect_origin(child)
                elif isinstance(value,list):
                    for child in value: inspect_origin(child)
            for seg in sm.get('segments',[]):
                start,end=seg.get('start'),seg.get('end')
                if not isinstance(start,int) or not isinstance(end,int) or start<0 or start>=end or end>len(mt):
                    fail(errors,f'API source-map UTF-8 text coordinates invalid for {key}: {start}/{end}')
                inspect_origin(seg.get('origin'))
        if isinstance(mt,str):
            cls=classify_text(mt); cls['quality']=quality
            derived.append({'name':key[0],'descriptor':key[1],**cls})
    if set(actual_methods)!=METHODS or len(actual_methods)!=len(METHODS): fail(errors,'API method identities differ from physical inventory')
    # Keep receipt summaries corroborative only; raw report is the source of derived classifications.
    expected_texts=cell.get('method_texts',[])
    expected_qualities=cell.get('method_qualities',[])
    if [m.get('outcome',{}).get('report',{}).get('text') for m in doc.get('methods',[])] != expected_texts:
        fail(errors,'receipt method text summaries differ from raw API report')
    if [m.get('outcome',{}).get('report',{}).get('quality') for m in doc.get('methods',[])] != expected_qualities:
        fail(errors,'receipt method quality summaries differ from raw API report')
    if [m.get('outcome',{}).get('report',{}).get('source_map') for m in doc.get('methods',[])] != cell.get('method_source_maps',[]):
        fail(errors,'receipt source-map summaries differ from raw API report')
    return {'doc':doc,'raw_classification':derived}
def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--execution',type=Path,required=True)
    ap.add_argument('--out',type=Path,required=True)
    args=ap.parse_args()
    errors=[]
    post_capture_source_drift=[]
    receipt_path=args.execution.resolve(strict=True)
    out=args.out.resolve()
    if out.exists(): raise SystemExit(f'refusing existing output {out}')
    receipt_bytes=receipt_path.read_bytes(); receipt_sha=sha_bytes(receipt_bytes)
    try: d=json.loads(receipt_bytes.decode('utf-8','strict'))
    except Exception as e: raise SystemExit(f'invalid execution JSON: {e}')
    if d.get('schema')!=SCHEMA: fail(errors,'unexpected receipt schema')
    if d.get('phase')!='baseline': fail(errors,'expected baseline phase; report separately')
    if d.get('status') not in ('captured-pending-review','failed'): fail(errors,'receipt did not reach a recognized terminal status')
    if d.get('status')=='captured-pending-review': note='receipt capture status is not product acceptance'
    else: note='capture failed; recorded evidence is partial and not product acceptance'
    commands=d.get('commands',[]); byindex={r.get('index'):r for r in commands}
    if len(byindex)!=len(commands) or sorted(byindex)!=list(range(1,len(commands)+1)): fail(errors,'command ledger indices are not contiguous/unique')
    command_labels={}
    for row in commands:
        idx=row.get('index'); label=row.get('label'); command_labels.setdefault(label,[]).append(idx)
        if not isinstance(row.get('argv'),list) or not row['argv'] or not all(isinstance(x,str) for x in row['argv']): fail(errors,f'command {idx} lacks exact argv')
        if row.get('cwd')!=d.get('root'): fail(errors,f'command {idx} cwd differs from repository root')
        for ch in ('stdout','stderr'): checked_stream(row,ch,errors)
        if row.get('guard_stop') is not None or row.get('peak_target_bytes',0)>1<<30 or row.get('free_bytes_after',0)<5*1024**3:
            fail(errors,f'command {idx} violated recorded resource guard')
    def one_command(index,label):
        row=byindex.get(index)
        if row is None: fail(errors,f'{label}: missing command index {index}'); return None
        if row.get('label')!=label: fail(errors,f'{label}: command label mismatch at index {index}')
        return row
    # Top-level input/tool pin and source closure checks.
    for field in ('cli','tool_reference_receipt','compare','guard','scanner','physical_parser','driver_source','target_source'):
        checked_pin(d.get(field),errors,f'receipt {field}')
    if d.get('tool_reference_receipt',{}).get('sha256')!=BASE_SHA: fail(errors,'original tool-reference receipt pin is not the reviewed baseline')
    if (d.get('target_source',{}).get('sha256'),d.get('driver_source',{}).get('sha256')) not in KNOWN_SOURCE_DRIVER_PAIRS:
        fail(errors,'target/driver source pins do not match any frozen v1-v4 private family')
    for toolfield in ('tool_reference_jadx','tool_reference_jadx_jar'):
        checked_pin(d.get(toolfield),errors,toolfield)
    if d.get('source_pins_before')!=d.get('source_pins_after') or d.get('source_unchanged_finally') is not True:
        fail(errors,'repository source closure changed or final source guard failed')
    for rel,pin in (d.get('source_pins_before') or {}).items():
        try:
            current=Path(pin['path']).read_bytes()
            if len(current)!=pin.get('bytes') or sha_bytes(current)!=pin.get('sha256'):
                post_capture_source_drift.append(rel)
        except Exception as e: post_capture_source_drift.append(rel+': '+str(e))
    if d.get('external_pins_before')!=d.get('external_pins_after') or d.get('external_inputs_unchanged_finally') is not True:
        fail(errors,'external tool/input pins changed or final external guard failed')
    for pin in d.get('external_pins_before') or []: checked_pin(pin,errors,'external input/tool')
    if d.get('local_copies_before')!=d.get('local_copies_after') or d.get('local_copies_unchanged_finally') is not True:
        fail(errors,'local frozen input copies changed')
    for name,pin in (d.get('local_copies_before') or {}).items(): checked_pin(pin,errors,'local input copy '+name)
    protected_before=d.get('generated_outputs_pins_before_final') or {}
    protected_after=d.get('generated_outputs_pins_after_final') or {}
    if protected_before!=protected_after or d.get('generated_outputs_unchanged_finally') is not True: fail(errors,'protected generated outputs changed')
    for name,pin in protected_before.items(): checked_pin(pin,errors,'protected generated output '+name)
    if d.get('target_absent_after_clean') is not True or d.get('target_absent_finally') is not True:
        fail(errors,'repository target was not absent after cleanup/finally')
    # Original compiler matrix and exact runtime oracles.
    source_profiles=d.get('source_profiles',[])
    if {x.get('profile') for x in source_profiles}!=set(PROFILES) or len(source_profiles)!=4: fail(errors,'original source compiler matrix incomplete')
    original_by_profile={}
    for row in source_profiles:
        profile=row.get('profile'); expected_label='compile-original-'+str(profile)
        cmd=one_command(row.get('command'),expected_label)
        if cmd and cmd.get('exit_code')!=row.get('exit_code'): fail(errors,f'{profile}: compile exit differs from command ledger')
        tree_names=pin_tree(row.get('class_tree'),errors,f'original class tree {profile}')
        root=Path(row.get('class_root',''))
        actual_names={p.relative_to(root).as_posix() for p in root.rglob('*.class')} if root.is_dir() else set()
        if actual_names!=tree_names: fail(errors,f'{profile}: actual original class tree differs from receipt')
        facts=row.get('class_facts') or {}
        if row.get('exit_code')==0:
            if tree_names!={INTERNAL+'.class'} or set(facts)!=tree_names: fail(errors,f'{profile}: original class closure incomplete')
            f=facts.get(INTERNAL+'.class',{})
            if f.get('class_name')!=INTERNAL or f.get('major')!=52 or f.get('access')!=0x31: fail(errors,f'{profile}: target class identity/version/access invalid')
            got={method_key(m) for m in f.get('methods',[])}
            if got!=METHODS: fail(errors,f'{profile}: physical methods differ')
            for m in f.get('methods',[]):
                code=m.get('code')
                if not code or not code.get('bcis') or code['bcis'][0]!=0 or code['bcis'][-1]>=code.get('code_length',0): fail(errors,f'{profile}: invalid physical Code BCIs for {method_key(m)}')
        original_by_profile[profile]=row
    original_oracle={}
    known_parser_corrections=[]
    runs=d.get('original_runs',[])
    if len(runs)!=8 or {(x.get('profile'),x.get('vm')) for x in runs}!={(p,v) for p in PROFILES for v in VMS}:
        fail(errors,'original runtime 4x2 matrix incomplete')
    for row in runs:
        profile,vm=row.get('profile'),row.get('vm'); cmd=one_command(row.get('command'),f'original-{profile}-{vm}')
        if cmd and cmd.get('exit_code')!=row.get('exit_code'): fail(errors,f'original runtime {profile}/{vm}: command exit mismatch')
        raw=checked_stream(cmd,'stdout',errors) if cmd else None
        if row.get('exit_code')==0 and raw is not None:
            parsed=parse_rows(raw,f'original {profile}/{vm}',errors)
            recorded=[x.get('raw') for x in row.get('rows',[])]
            if parsed!=recorded:
                if not recorded and KNOWN_COLLECTOR_PARSE_ERROR in str(row.get('parse_error')):
                    known_parser_corrections.append({'kind':'original-runtime-row-parser','profile':profile,'vm':vm,
                        'collector_parse_error':row.get('parse_error'),'raw_rows_reconstructed':len(parsed)})
                else: fail(errors,f'original {profile}/{vm}: receipt rows differ from stdout outside the known case-name regex bug')
            original_oracle[(profile,vm)]=parsed
        else: fail(errors,f'original runtime failed: {profile}/{vm} exit={row.get("exit_code")}')
    wrong_original=d.get('original_wrong_root',[])
    if len(wrong_original)!=4 or {r.get('profile') for r in wrong_original}!=set(PROFILES): fail(errors,'original wrong-root profile matrix incomplete')
    for row in wrong_original:
        cmd=one_command(row.get('command'),f'original-wrong-root-{row.get("profile")}')
        err=checked_stream(cmd,'stderr',errors) if cmd else None
        rejected=bool(cmd and cmd.get('exit_code')==1 and err is not None and b'wrong CodeSource:' in err)
        if row.get('exit_code')!=(cmd or {}).get('exit_code') or row.get('rejected') is not rejected: fail(errors,'original wrong-root result differs from raw command')
    # API grid and raw-source-map-backed method classification.
    api_rows=d.get('jarde_cells',[])
    if len(api_rows)!=12 or {(x.get('profile'),x.get('evidence')) for x in api_rows}!={(p,e) for p in PROFILES for e in EVIDENCE}:
        fail(errors,'Jarde API profile/evidence grid incomplete')
    raw_classifications=[]
    api_sources={}
    for row in api_rows:
        profile,evidence=row.get('profile'),row.get('evidence')
        cmd=one_command(row.get('command'),f'jarde-{profile}-{evidence}')
        if cmd and cmd.get('exit_code')!=row.get('exit_code'): fail(errors,'API exit differs from command ledger')
        if cmd:
            stdout=checked_stream(cmd,'stdout',errors)
            if stdout is not None and row.get('report',{}).get('sha256')!=sha_bytes(stdout): fail(errors,'API report pin does not match exact stdout bytes')
        audited=source_class(d,row,errors)
        if audited:
            raw_classifications.append({'profile':profile,'evidence':evidence,'methods':audited['raw_classification']})
            api_sources[(profile,evidence)]=row
            if row.get('method_inventory_matches_source') is not True: fail(errors,'API summary method inventory flag false')
    # Complete JADX source closure is established by byte-identical class archive membership.
    jadx_rows=d.get('jadx_cells',[])
    if len(jadx_rows)!=4 or {r.get('profile') for r in jadx_rows}!=set(PROFILES): fail(errors,'JADX profile matrix incomplete')
    for row in jadx_rows:
        profile=row.get('profile'); cmd=one_command(row.get('command'),f'jadx-{profile}')
        if cmd and cmd.get('exit_code')!=row.get('exit_code'): fail(errors,'JADX exit differs from command ledger')
        bundle=row.get('input_bundle'); jarbytes=checked_pin(bundle,errors,f'JADX bundle {profile}')
        srcprof=original_by_profile.get(profile,{})
        tree=srcprof.get('class_tree',{})
        if jarbytes is not None:
            try:
                import io
                with zipfile.ZipFile(io.BytesIO(jarbytes)) as z:
                    members=sorted(n for n in z.namelist() if n.endswith('.class'))
                    if members!=sorted(tree): fail(errors,f'{profile}: JADX bundle class membership differs')
                    for name in members:
                        raw=z.read(name); pin=tree.get(name,{})
                        if len(raw)!=pin.get('bytes') or sha_bytes(raw)!=pin.get('sha256'): fail(errors,f'{profile}: JADX bundle class bytes differ {name}')
            except Exception as e: fail(errors,f'{profile}: invalid JADX class bundle: {e}')
        source_pins=row.get('sources',[])
        if len(source_pins)!=row.get('source_count'): fail(errors,f'{profile}: JADX source count differs from source closure')
        for source_pin in source_pins: checked_pin(source_pin,errors,f'JADX source {profile}')
        if row.get('exit_code')==0 and (not source_pins or row.get('complete_source_closure') is not True):
            fail(errors,f'{profile}: successful JADX run lacks complete source closure')
    # Derived generated compile selectors; every successful target compile gets exactly 2 VMs + wrong-root.
    expected_selectors=set()
    for row in api_rows:
        if row.get('source_ready'):
            for compiler in VMS: expected_selectors.add(('jarde',row['profile'],row['evidence'],compiler))
    for row in jadx_rows:
        if row.get('sources'):
            for compiler in VMS: expected_selectors.add(('jadx',row['profile'],'source',compiler))
    compiles=d.get('generated_compiles',[])
    compile_selector=lambda r:(r.get('kind'),r.get('profile'),r.get('variant'),r.get('compiler'))
    if {compile_selector(r) for r in compiles}!=expected_selectors or len(compiles)!=len(expected_selectors):
        fail(errors,'generated compile selectors do not match complete API/JADX source closures')
    success_keys=set(); compile_results=[]
    for row in compiles:
        sel=compile_selector(row); label='compile-'+row.get('kind')+'-'+row.get('profile')+'-'+row.get('variant')+'-'+row.get('compiler')+'-'+row.get('debug','').lstrip('-')
        cmd=one_command(row.get('command'),label)
        if cmd and cmd.get('exit_code')!=row.get('exit_code'): fail(errors,f'generated compile exit mismatch {sel}')
        copied=row.get('source_copies',[])
        if not copied: fail(errors,f'generated compile has no source copy inventory {sel}')
        for pair in copied:
            source=checked_pin(pair.get('source'),errors,f'generated source origin {sel}')
            copy=checked_pin(pair.get('copy'),errors,f'generated source copy {sel}')
            if source is None or copy is None or source!=copy: fail(errors,f'generated source copy bytes differ {sel}')
            name=Path(pair.get('copy',{}).get('path','')).name
            if not re.fullmatch(r'[A-Za-z_$][A-Za-z0-9_$]*\.java',name): fail(errors,f'illegal generated Java filename {name}')
        tree_names=pin_tree(row.get('class_tree'),errors,f'generated class tree {sel}')
        root=Path(row.get('class_root',''))
        actual_names={p.relative_to(root).as_posix() for p in root.rglob('*.class')} if root.is_dir() else set()
        if actual_names!=tree_names: fail(errors,f'actual generated class tree differs from receipt {sel}')
        if row.get('exit_code')==0:
            if row.get('class_inventory_matches_original') is not True: fail(errors,f'generated class inventory differs from original {sel}')
            if tree_names!={INTERNAL+'.class'}: fail(errors,f'generated target class closure incomplete {sel}')
            success_keys.add(sel)
        else:
            if tree_names: # partial classes on failed javac are preserved as a failure, never as runtime success
                pass
        compile_results.append({'selector':sel,'exit_code':row.get('exit_code'),'inventory_matches_original':row.get('class_inventory_matches_original')})
    generated_runs=d.get('generated_runs',[]); wrong_gen=d.get('generated_wrong_root',[])
    generated_keys=lambda r:(r.get('kind'),r.get('profile'),r.get('variant'),r.get('compiler'))
    expected_runs={( *sel, vm) for sel in success_keys for vm in VMS}
    actual_runs={( *generated_keys(r), r.get('vm')) for r in generated_runs}
    if actual_runs!=expected_runs or len(generated_runs)!=len(expected_runs): fail(errors,'generated runtime rows do not cover exactly 2 VMs per successful compile')
    expected_wrong=success_keys
    actual_wrong={generated_keys(r) for r in wrong_gen}
    if actual_wrong!=expected_wrong or len(wrong_gen)!=len(expected_wrong): fail(errors,'generated wrong-root rows do not cover every successful compile')
    runtime_results=[]
    for row in generated_runs:
        sel=generated_keys(row); vm=row.get('vm'); cmd=one_command(row.get('command'),f'generated-{"-".join(map(str,sel))}-{row.get("debug","").lstrip("-")}-{vm}')
        if cmd and cmd.get('exit_code')!=row.get('exit_code'): fail(errors,f'generated runtime exit mismatch {sel}/{vm}')
        raw=checked_stream(cmd,'stdout',errors) if cmd else None
        computed=False
        if cmd and cmd.get('exit_code')==0 and raw is not None:
            parsed=parse_rows(raw,f'generated {sel}/{vm}',errors)
            receipt_rows=[x.get('raw') for x in row.get('rows',[])]
            if parsed!=receipt_rows:
                if not receipt_rows and KNOWN_COLLECTOR_PARSE_ERROR in str(row.get('parse_error')):
                    known_parser_corrections.append({'kind':'generated-runtime-row-parser','selector':list(sel),'vm':vm,
                        'collector_parse_error':row.get('parse_error'),'raw_rows_reconstructed':len(parsed)})
                else: fail(errors,f'generated receipt rows differ from stdout {sel}/{vm} outside the known case-name regex bug')
            expected=original_oracle.get((row.get('profile'),vm))
            computed=expected is not None and parsed==expected
        if row.get('matches_original') is not computed:
            if KNOWN_COLLECTOR_PARSE_ERROR in str(row.get('parse_error')):
                known_parser_corrections.append({'kind':'generated-runtime-oracle-flag','selector':list(sel),'vm':vm,
                    'collector_matches_original':row.get('matches_original'),'raw_matches_original':computed})
            else: fail(errors,f'generated matches_original flag differs from raw stdout {sel}/{vm}')
        runtime_results.append({'selector':sel,'vm':vm,'exit_code':row.get('exit_code'),'matches_original_from_stdout':computed})
    for row in wrong_gen:
        sel=generated_keys(row); cmd=one_command(row.get('command'),f'generated-wrong-root-{"-".join(map(str,sel))}-{row.get("debug","").lstrip("-")}')
        err=checked_stream(cmd,'stderr',errors) if cmd else None
        rejected=bool(cmd and cmd.get('exit_code')==1 and err is not None and b'wrong CodeSource:' in err)
        if row.get('exit_code')!=(cmd or {}).get('exit_code') or row.get('rejected') is not rejected:
            fail(errors,f'generated wrong-root flag differs from raw stderr {sel}')
    # Recompute exact failures from evidence; summarize without suppressing any error class.
    computed_failures=[]
    driver_compile=d.get('driver_compile') or {}
    if driver_compile and driver_compile.get('exit_code')!=0:
        computed_failures.append({'kind':'driver-compile','selector':[],'exit_code':driver_compile.get('exit_code')})
    for row in source_profiles:
        if row.get('exit_code')!=0: computed_failures.append({'kind':'original-compile','selector':[row.get('profile')],'exit_code':row.get('exit_code')})
    for row in api_rows:
        if row.get('exit_code')!=0 or row.get('source_ready') is not True: computed_failures.append({'kind':'api','selector':[row.get('profile'),row.get('evidence')],'exit_code':row.get('exit_code')})
    for row in jadx_rows:
        if row.get('exit_code')!=0 or not row.get('sources') or row.get('complete_source_closure') is not True:
            computed_failures.append({'kind':'jadx','selector':[row.get('profile')],'exit_code':row.get('exit_code')})
    for row in compiles:
        if row.get('exit_code')!=0 or row.get('class_inventory_matches_original') is not True:
            computed_failures.append({'kind':'compile','selector':list(compile_selector(row)),'exit_code':row.get('exit_code')})
    raw_runtime_status={(tuple(item['selector']),item['vm']):(item['exit_code'],item['matches_original_from_stdout']) for item in runtime_results}
    for row in generated_runs:
        raw_exit,raw_match=raw_runtime_status.get((tuple(generated_keys(row)),row.get('vm')),(row.get('exit_code'),False))
        if raw_exit!=0 or raw_match is not True:
            computed_failures.append({'kind':'runtime','selector':[*generated_keys(row),row.get('vm')],
                'exit_code':raw_exit,'matches_original_from_raw':raw_match})
    for row in wrong_gen:
        if row.get('rejected') is not True:
            computed_failures.append({'kind':'wrong-root','selector':list(generated_keys(row)),'exit_code':row.get('exit_code')})
    for row in runs:
        if row.get('exit_code')!=0 or (row.get('profile'),row.get('vm')) not in original_oracle:
            computed_failures.append({'kind':'original-runtime','selector':[row.get('profile'),row.get('vm')],'exit_code':row.get('exit_code')})
    for row in wrong_original:
        if row.get('rejected') is not True: computed_failures.append({'kind':'original-wrong-root','selector':[row.get('profile')],'exit_code':row.get('exit_code')})
    # Protected-method coverage is derived from exact raw report text per API cell.
    protected=('protectedPositive','protectedAfterWrite','protectedBeforeRead','protectedAfterRead','protectedCatchReadWrite')
    protected_coverage=[]
    for item in raw_classifications:
        for method in item['methods']:
            if method['name'] in protected:
                seen=method['foreach_headers']>0
                clean=seen and method['quality']=='structured' and not method['blank_local_declarations']
                protected_coverage.append({'profile':item['profile'],'evidence':item['evidence'],'method':method['name'],
                    'foreach_headers':method['foreach_headers'],'quality':method['quality'],
                    'blank_local_declarations':method['blank_local_declarations'],
                    'observed_foreach':seen,'counts_as_clean_foreach_coverage':clean})
    outrec={
        'schema':'cf11-iterator-ancestor-controls-independent-audit-v1',
        'execution_receipt':{'path':str(receipt_path),'bytes':len(receipt_bytes),'sha256':receipt_sha,'captured_status':d.get('status')},
        'audit_status':'integrity-check-failed' if errors else 'integrity-consistent',
        'product_acceptance':'not-assessed-by-this-audit',
        'note':note,
        'command_count':len(commands),'command_labels':{k:v for k,v in sorted(command_labels.items())},
        'matrix_counts':{'original_profiles':len(source_profiles),'original_vm_runs':len(runs),'api_cells':len(api_rows),
            'jadx_cells':len(jadx_rows),'generated_compiles':len(compiles),'successful_generated_compiles':len(success_keys),
            'generated_vm_runs':len(generated_runs),'generated_wrong_root_runs':len(wrong_gen)},
        'compile_results':compile_results,'generated_runtime_results':runtime_results,
        'raw_method_classifications':raw_classifications,'protected_method_foreach_coverage':protected_coverage,
        'computed_failed_cells':computed_failures,'collector_failed_cells':d.get('failed_cells',[]),
        'collector_failure_list_matches_recomputed':None,
        'collector_failure_list_note':'The collector regex rejects the uppercase N in protected-hasNext-throws; raw stdout was independently reparsed. The original failure list is preserved but not treated as oracle.',
        'known_parser_corrections':known_parser_corrections,
        'integrity_errors':errors,
        'source_unchanged_finally':d.get('source_unchanged_finally'),
        'post_capture_source_drift':post_capture_source_drift,
        'external_inputs_unchanged_finally':d.get('external_inputs_unchanged_finally'),
        'generated_outputs_unchanged_finally':d.get('generated_outputs_unchanged_finally'),
        'target_absent_after_clean':d.get('target_absent_after_clean'),
    }
    outrec['integrity_errors']=errors
    outrec['audit_status']='integrity-check-failed' if errors else 'integrity-consistent'
    out.mkdir(parents=True)
    output=out/'review.json'
    output.write_text(json.dumps(outrec,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'audit_status':outrec['audit_status'],'product_acceptance':outrec['product_acceptance'],
        'receipt_sha256':receipt_sha,'command_count':len(commands),'matrix_counts':outrec['matrix_counts'],
        'computed_failure_count':len(computed_failures),'integrity_error_count':len(errors),'review':str(output)},ensure_ascii=False))
    return 0 if not errors else 1
if __name__=='__main__': raise SystemExit(main())
