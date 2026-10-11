#!/usr/bin/env python3
"""Independent read-only acceptance for the prepared CF07 final replay."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import struct

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
CLI_PATH = Path('/private/tmp/jarde-computed-for-candidate-cli-root-v1')
RUNNER_PATH = Path('/private/tmp/jarde-computed-for-cf07-final-jdk-replay-luna-v2/replay.py')
RUNNER_SHA = 'cf374758e3f9710735fe267b63a3da6b85136a240af849496da0a993f8dc9174'
BUILD_SCHEMA = 'jarde-final-cli-build-clean-root-v1'
RUN_SCHEMA = 'cf07-final-dual-jdk-full-source-replay-luna-v2'
JADX_STATUS = 'passed_fresh_JADX_full_LoopCases_both_JDKs'
JADX_SOURCE_SHA = '4d5023782e6d10dae9bd8b786ad18ec62d8029e08cea9eebec38f8036e815fc2'
EXPECTED = {
    'original_source': ('openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/original-sources/LoopCases.java', '3184f43aa78ba2f26752d15e0b706fe7b0cc9cf25bca34dded34bf603ecb3c38'),
    'original_runner': ('openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/original-sources/Runner.java', '28b0a08ceb89b6ff31d12056831a2d7cc62bfda0a61ebc39835d2f62bc89bdf5'),
    'jadx_source': ('openspec/changes/recover-proved-computed-for-initializers/results/complete-controls-root-v1/fresh-cf07-jadx/out/sources/cf07/LoopCases.java', JADX_SOURCE_SHA),
    'jadx_execution_record': ('openspec/changes/recover-proved-computed-for-initializers/results/complete-controls-root-v1/fresh-cf07-jadx/execution.json', None),
    'guard_template': ('openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py', '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'),
    'target_size_scanner': ('openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py', 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'),
}
JDKS = {
    'javac8': {
        'home': Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
        'javac': '598601455e9f81ef05225c014743aaafa993d572afc7f83f5e7934ff10f9dddd',
        'java': '3167d94b1b56e1330e25e48a92bba9cba423ca578525c4ae66e8b5a06fb5dbb5',
        'class': ('openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/cases/javac8-original/classes/cf07/LoopCases.class', 'eeda5de66a2f436f077ef3fcf852fe054c01e1a86b71b5638777ce5a0b0a9708'),
    },
    'javac23': {
        'home': Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'),
        'javac': 'a3e79462d70cb70c34b85ab94ae615328d1cbe7ac6a2c69635288ae6cbb902a8',
        'java': 'b79b8bac2b2a2c2b4d0dcb9c3981d477bd58bc95c5c4404dfb40a37e581497a1',
        'class': ('openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/cases/javac23-original/classes/cf07/LoopCases.class', '004e99baeb062861f95a4dac13e3a78399a66efd31f2589b0028cfe50432ae33'),
    },
}
LAST_BCIS = [0, 1, 2, 3, 5, 7, 8, 11, 12, 14, 15, 16, 19, 21, 22, 25, 28, 29]
COUNTED_BCIS = [0, 1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 14, 17, 20, 23, 24, 25, 26, 27, 30, 33, 36, 37]


def need(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_pin(path):
    path = Path(path)
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': sha(data)}


def check_pin(record, label):
    actual = file_pin(record['path'])
    need(actual['bytes'] == record['bytes'] and actual['sha256'] == record['sha256'], f'{label}: live pin mismatch {actual}')
    return actual


def u1(data, at):
    return data[at]


def u2(data, at):
    return struct.unpack_from('>H', data, at)[0]


def u4(data, at):
    return struct.unpack_from('>I', data, at)[0]


def parse_method(data, wanted_name, wanted_desc):
    need(data[:4] == b'\xca\xfe\xba\xbe', 'bad class-file magic')
    cp_count = u2(data, 8)
    cp = [None] * cp_count
    at, index = 10, 1
    while index < cp_count:
        tag = u1(data, at); at += 1
        if tag == 1:
            length = u2(data, at); at += 2
            cp[index] = (tag, data[at:at + length].decode('utf-8')); at += length
        elif tag in (3, 4): cp[index] = (tag, data[at:at + 4]); at += 4
        elif tag in (5, 6): cp[index] = (tag, data[at:at + 8]); at += 8; index += 1
        elif tag in (7, 8, 16, 19, 20): cp[index] = (tag, u2(data, at)); at += 2
        elif tag in (9, 10, 11, 12, 17, 18): cp[index] = (tag, u2(data, at), u2(data, at + 2)); at += 4
        elif tag == 15: cp[index] = (tag, u1(data, at), u2(data, at + 1)); at += 3
        else: raise ValueError(f'unsupported constant-pool tag {tag}')
        index += 1

    def utf8(i):
        item = cp[i]; need(item is not None and item[0] == 1, f'constant {i} is not UTF8'); return item[1]
    def class_name(i):
        item = cp[i]; need(item is not None and item[0] == 7, f'constant {i} is not Class'); return utf8(item[1])
    owner = class_name(u2(data, at + 2)); at += 6
    interfaces = u2(data, at); at += 2 + 2 * interfaces
    def skip_members(pos, count):
        for _ in range(count):
            attrs = u2(data, pos + 6); pos += 8
            for _ in range(attrs): pos += 6 + u4(data, pos + 2)
        return pos
    fields = u2(data, at); at = skip_members(at + 2, fields)
    count = u2(data, at); at += 2
    found = None
    for _ in range(count):
        name, desc, attrs = utf8(u2(data, at + 2)), utf8(u2(data, at + 4)), u2(data, at + 6)
        pos = at + 8
        for _ in range(attrs):
            attr_name, length = utf8(u2(data, pos)), u4(data, pos + 2)
            payload = pos + 6
            if name == wanted_name and desc == wanted_desc and attr_name == 'Code':
                code_len = u4(data, payload + 4); code_start = payload + 8
                exc_count = u2(data, code_start + code_len)
                found = {'owner': owner, 'code': data[code_start:code_start + code_len], 'exception_count': exc_count}
            pos += 6 + length
        at = pos
    need(found is not None, f'missing Code for {wanted_name}{wanted_desc}')
    return found


def insn_size(code, pc):
    op = code[pc]
    if op in (0x10, 0x12, 0x15, 0x16, 0x17, 0x18, 0x19, 0x36, 0x37, 0x38, 0x39, 0x3a, 0xa9, 0xbc): return 2
    if op in (0x11, 0x13, 0x14, 0x84, *range(0x99, 0xa9), 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xbb, 0xbd, 0xc0, 0xc1, 0xc6, 0xc7): return 3
    if op in (0xb9, 0xba): return 5
    if op == 0xc5: return 4
    if op in (0xc8, 0xc9): return 5
    if op == 0xaa:
        base = pc + 1 + ((4 - ((pc + 1) & 3)) & 3); low, high = struct.unpack_from('>ii', code, base + 4)
        return base - pc + 12 + 4 * (high - low + 1)
    if op == 0xab:
        base = pc + 1 + ((4 - ((pc + 1) & 3)) & 3); pairs = struct.unpack_from('>i', code, base + 4)[0]
        return base - pc + 8 + 8 * pairs
    if op == 0xc4: return 6 if code[pc + 1] == 0x84 else 4
    return 1


def decode(code):
    out, pc = [], 0
    while pc < len(code):
        size = insn_size(code, pc)
        need(size > 0 and pc + size <= len(code), f'malformed instruction at BCI {pc}')
        out.append((pc, code[pc], code[pc:pc + size])); pc += size
    need(pc == len(code), 'instruction decoder did not reach Code end')
    return out


def report_for(doc, name, desc):
    matches = []
    for item in doc.get('methods', []):
        report = item.get('outcome', {}).get('report') if isinstance(item.get('outcome'), dict) else None
        if isinstance(report, dict) and report.get('method') == name + desc:
            matches.append(report)
    need(len(matches) == 1, f'class report must contain exactly one {name}{desc} report')
    return matches[0]


def all_origins(report):
    for segment in report.get('source_map', {}).get('segments', []):
        origin = segment.get('origin', {})
        primary = origin.get('primary')
        if primary is not None: yield segment, primary
        for fact in origin.get('derived', []): yield segment, fact


def span_bytes(report, segment):
    text = report['text'].encode('utf-8')
    start, end = segment['start'], segment['end']
    need(isinstance(start, int) and isinstance(end, int) and 0 <= start < end <= len(text), 'source-map span escapes method report')
    text[:start].decode('utf-8'); text[:end].decode('utf-8')
    return text[start:end].decode('utf-8')


def verify_method_map(doc, method_report, class_path, name, desc, expected_bcis):
    data = Path(class_path).read_bytes()
    physical = parse_method(data, name, desc)
    insns = decode(physical['code'])
    bcis = [row[0] for row in insns]
    need(bcis == expected_bcis, f'{name}: physical instruction BCI list differs: {bcis}')
    need(physical['exception_count'] == 0, f'{name}: unexpected exception handler in frozen method')
    class_identity = doc['class']
    class_bytes = class_identity['class_bytes']
    need(class_bytes['length'] == len(data), f'{name}: report class length differs from input')
    need(re.fullmatch(r'[0-9a-f]{64}', class_bytes['digest']) is not None, f'{name}: report class digest is not a 32-byte hex identity')
    need(class_identity['location'].get('kind') == 'standalone_root' and class_identity['variant'].get('kind') == 'base', f'{name}: unexpected physical class location/variant')
    seen = set(); rows = []
    for seg, origin in all_origins(method_report):
        _ = span_bytes(method_report, seg)
        need(origin.get('method') is not None, f'{name}: origin omits physical method identity')
        method = origin['method']
        need(bytes(method['name']).decode('ascii') == name, f'{name}: origin has a different UTF8 method name')
        need(bytes(method['descriptor']).decode('ascii') == desc, f'{name}: origin has a different UTF8 descriptor')
        need(method['owner'] == class_identity, f'{name}: source-map owner differs from rendered class physical identity')
        need(origin['bci'] in bcis, f'{name}: source map contains non-instruction BCI {origin["bci"]}')
        seen.add(origin['bci']); rows.append((seg, origin))
    need(seen == set(bcis), f'{name}: physical source map differs; missing={sorted(set(bcis)-seen)}, extra={sorted(seen-set(bcis))}')
    return data, physical, rows


def spans_for(rows, bci, provenance=None):
    found=[]
    for seg, origin in rows:
        if origin['bci'] == bci and (provenance is None or origin.get('provenance') == provenance):
            found.append(seg)
    return found


def line_start(text, offset):
    return text.rfind('\n', 0, offset) + 1


def statement_body(text, marker, at=0):
    start = text.find(marker, at)
    need(start >= 0, f'missing statement marker {marker!r}')
    open_brace = text.find('{', start)
    need(open_brace >= 0, f'missing body opening brace after {marker!r}')
    depth = 0
    for pos in range(open_brace, len(text)):
        if text[pos] == '{': depth += 1
        elif text[pos] == '}':
            depth -= 1
            if depth == 0:
                end = pos + 1
                if end < len(text) and text[end] == '\n': end += 1
                return start, end, open_brace + 1, pos
    raise ValueError(f'unclosed body for {marker!r}')


def exact_origin_span(rows, bci, start, end, provenance='Derived'):
    matches=[s for s in spans_for(rows,bci,provenance) if s['start']==start and s['end']==end]
    need(len(matches)==1, f'BCI {bci} must derive exactly one requested complete span {start}..{end}, found {len(matches)}')


def verify_loop_semantics(doc, class_path, method_name, report, mode):
    if method_name == 'lastIndexOf':
        desc='([IIII)I'; want=[0,1,2,3,5,7,8,11,12,14,15,16,19,21,22,25,28,29]
        data, physical, rows=verify_method_map(doc,report,class_path,method_name,desc,want)
        text=report['text']; header_at=text.find('for ('); need(header_at>=0,'lastIndexOf did not render a for header')
        start=line_start(text,header_at); fs, end, body_start, close=statement_body(text,'for (')
        need(fs==header_at,'for parser lost header')
        header=text[header_at:body_start-1]
        need(header.count(';')==2,'lastIndexOf for header lacks init/condition/update clauses')
        need('local4 = arg3 - 1;' in header and 'local4 >= arg2;' in header and 'local4 = local4 - 1' in header,
             f'lastIndexOf for init/condition/update differs: {header}')
        loop_end=end
        exact_origin_span(rows,25,start,loop_end)
        need(not text[start:loop_end].count('return -1;') and 'return -1;' in text[loop_end:],
             'lastIndexOf for span must exclude the later -1 return')
        for bci in (0,1,2,3,8,22):
            need(any(start <= seg['start'] < seg['end'] <= close for seg in spans_for(rows,bci)),
                 f'lastIndexOf physical BCI {bci} is not anchored inside the for header/body')
        for bci in (28,29):
            need(any(seg['start'] >= loop_end for seg in spans_for(rows,bci)),
                 f'lastIndexOf tail BCI {bci} is not after the complete for span')
        owners=[r for r in report['regions'] if all(b in r['blocks'] for b in (5,11,19,22))]
        need((len(owners)==1 and owners[0]['structured'] and owners[0]['rule']['rule']=='loop') if mode=='all' else report['regions']==[],
             'lastIndexOf loop blocks lack one structured loop owner')
        return {'method':method_name+desc,'physical_bcis':want,'for_span_bytes':[start,loop_end],
                'for_header':header,'init_bcis':[0,1,2,3],'condition_bci':8,'update_bci':22,'latch_bci':25,
                'tail_return_excluded':True,'loop_owner_blocks':owners[0]['blocks'] if mode=='all' else None}
    desc='(II)I'; want=[0,1,2,3,4,5,6,7,8,11,12,14,17,20,23,24,25,26,27,30,33,36,37]
    data, physical, rows=verify_method_map(doc,report,class_path,method_name,desc,want)
    text=report['text']; header_at=text.find('for ('); need(header_at>=0,'counted did not render a for header')
    start=line_start(text,header_at); fs, end, body_start, close=statement_body(text,'for (')
    header=text[header_at:body_start-1]
    need(header.count(';')==2,'counted for header lacks init/condition/update clauses')
    need('local3 = arg0;' in header and 'local3 < arg1;' in header and 'local3 = local3 + 1' in header,
         f'counted for init/condition/update differs: {header}')
    for bci in (4,5,8,27):
        need(any(start <= seg['start'] < seg['end'] <= close for seg in spans_for(rows,bci)),
             f'counted physical BCI {bci} is not anchored in the for statement')
    # The complete inner If is one exact derived span from goto@20.
    if_at=text.find('if ('); need(if_at>=0 and if_at < close,'counted has no inner If inside for body')
    if_start=line_start(text,if_at)
    then_start, then_end, _, _=statement_body(text,'if (',if_at)
    else_at=text.find('else {',then_end)
    need(else_at>=0 and else_at < close,'counted If has no else arm inside loop')
    _, if_end, _, _=statement_body(text,'else {',else_at)
    exact_origin_span(rows,20,if_start,if_end)
    need(start <= if_start < if_end <= close,'counted complete If span escapes for body')
    exact_origin_span(rows,30,start,end)
    owners=[r for r in report['regions'] if all(b in r['blocks'] for b in (6,11,17,23,27))]
    need((len(owners)==1 and owners[0]['structured'] and owners[0]['rule']['rule']=='loop') if mode=='all' else report['regions']==[],
         'counted loop blocks lack one structured loop owner')
    return {'method':method_name+desc,'physical_bcis':want,'for_span_bytes':[start,end],
            'for_header':header,'init_bcis':[4,5],'condition_bci':8,'update_bci':27,
            'complete_if_span_bytes':[if_start,if_end],'if_join_goto_bci':20,'latch_bci':30,
            'loop_owner_blocks':owners[0]['blocks'] if mode=='all' else None}


def verify_raw(command, run_dir):
    need(command['exit_code']==0 and command['guard_stop'] is None, f'command {command["index"]} failed/guard-stopped')
    need(command['peak_target_bytes'] <= 1073741824 and command['free_bytes_after'] >= 5368709120,
         f'command {command["index"]} violates recorded resource guard')
    output={}
    for name in ('stdout','stderr'):
        rec=command['streams'][name]
        expected=run_dir/f'{command["index"]}.{name}.raw'
        need(Path(rec['path']).resolve()==expected.resolve(), f'command {command["index"]} {name} path is not index-bound raw')
        data=expected.read_bytes()
        need(len(data)==rec['bytes'] and sha(data)==rec['sha256'],f'command {command["index"]} {name} raw pin mismatch')
        output[name]=data
    return output


def verify_build(build_path, cli_path, cli_sha, workspace):
    raw=build_path.read_bytes(); build=json.loads(raw)
    need(build.get('schema')==BUILD_SCHEMA and build.get('status')=='validation-passed-cli-frozen-target-cleaned','final CLI build receipt not accepted')
    need(build.get('target_absent_after_clean') is True and build.get('pre_build_target_absent') is True and build.get('failure') is None,'build receipt does not prove target absent/clean success')
    need(len(build.get('commands',[]))==2 and all(c['exit_code']==0 and c['guard_stop'] is None for c in build['commands']),'build/clean command pair failed')
    need(build.get('guard_limits')=={'free_bytes':5368709120,'target_bytes':1073741824},'unexpected CLI build guard')
    need(build.get('source_paths_before')==build.get('source_paths_after'),'source path closure changed during CLI build')
    before=build.get('source_pins_before',{}); after=build.get('source_pins_after',{})
    need(before==after,'source closure pins differ before/after CLI build')
    full=before.get('full_product_source_and_include_closure')
    need(isinstance(full,dict) and full==build.get('freeze',{}).get('product_source_pins'),'frozen product closure differs')
    need(sorted(full)==build['source_paths_before'],'source path list and full closure keys differ')
    need(len(full)==build.get('product_source_pin_count'),'dynamic product source pin count differs')
    live=0
    for rel,digest in full.items():
        path=workspace/rel; data=path.read_bytes()
        need(sha(data)==digest,f'live product source changed: {rel}')
        live+=1
    frozen=build['freeze']['frozen_cli']
    built=build['freeze']['built_binary']
    need(built['sha256']==frozen['sha256'] and built['size_bytes']==frozen['size_bytes'],'built and frozen CLI identities differ')
    need(Path(frozen['path']).resolve()==cli_path.resolve()==CLI_PATH.resolve(),'frozen CLI path differs')
    data=cli_path.read_bytes(); mode=stat.S_IMODE(cli_path.stat().st_mode)
    need(len(data)==frozen['size_bytes'] and sha(data)==frozen['sha256']==cli_sha,'CLI path/size/SHA differs from receipt/argv')
    need(mode==0o555 and int(frozen['mode'],8)==mode,'CLI mode differs from frozen receipt')
    build_streams=[]
    for row in build['commands']:
        for stream in ('stdout','stderr'):
            pin= row['streams'][stream]; b=Path(pin['path']).read_bytes()
            need(len(b)==pin['bytes'] and sha(b)==pin['sha256'],f'build command raw changed: {row["index"]}/{stream}')
            build_streams.append(pin)
    return build, {'schema':build['schema'],'status':build['status'],'cli':{'path':str(cli_path),'bytes':len(data),'sha256':sha(data),'mode':oct(mode)},
                   'live_product_source_pin_count':live,'product_source_closure_equal_before_after_freeze':True,'build_raw_streams':len(build_streams)}


def expected_command_labels():
    labels=['jarde-render-javac8-default','jarde-render-javac8-all','jarde-render-javac23-default','jarde-render-javac23-all']
    for jdk in ('javac8','javac23'):
        labels += [f'compile-original-runner-{jdk}',f'run-original-input-class-{jdk}']
        for variant in ('original-source','jadx-frozen','jarde-default','jarde-all'):
            labels += [f'compile-{variant}-{jdk}',f'run-{variant}-{jdk}']
    return labels


def verify_execution(execution_path, build_path, cli_path, cli_sha, out_path, workspace):
    raw_exec=execution_path.read_bytes(); run=json.loads(raw_exec)
    need(run.get('schema')==RUN_SCHEMA and run.get('status')=='passed_all_sources_both_jdks_against_original_class_oracles','replay not fully accepted')
    need(len(run.get('commands',[]))==24,'replay must contain exactly 24 commands')
    commands=run['commands']; need([c['index'] for c in commands]==list(range(100,124)),'command indexes must be exact 100..123')
    need([c['label'] for c in commands]==expected_command_labels(),'command order/labels differ from expected four renders plus two ten-command JDK legs')
    run_dir=execution_path.resolve().parent
    stream_data={c['index']:verify_raw(c,run_dir) for c in commands}
    build,build_summary=verify_build(build_path,cli_path,cli_sha,workspace)
    need(run['cli']['path']==str(cli_path.resolve()) and run['cli']['sha256']==cli_sha,'replay CLI binding differs')
    need(run['cli']['freeze']==build['freeze'] and run['cli']['build_status']==build['status'],'replay did not bind exact build freeze')
    need(run['cli']['build_receipt']==file_pin(build_path),'replay build receipt pin differs')
    need(run['cli']['freeze']['product_source_pins']==build['freeze']['product_source_pins'],'replay full source closure differs from build receipt')
    need(run.get('scope_note','').find('does not invoke JADX')>=0,'replay incorrectly claims a fresh JADX invocation')
    need(run['runner']==file_pin(RUNNER_PATH),'replay runner bytes/hash differ from frozen v2 runner')
    need(run['runner']['sha256']==RUNNER_SHA,'unexpected replay runner hash')
    for name,(relative,expected) in EXPECTED.items():
        rec=run['inputs'][name]; actual=file_pin(workspace/relative)
        need(rec==actual,f'replay input pin mismatch: {name}')
        if expected is not None: need(actual['sha256']==expected,f'{name} differs from frozen baseline')
    need(len(run['inputs'])==len(EXPECTED)+2,'replay input inventory has unexpected/missing items')
    for jdk, facts in JDKS.items():
        input_name=f'original_{jdk}_class'; rel,expected=facts['class']; actual=file_pin(workspace/rel)
        need(run['inputs'][input_name]==actual and actual['sha256']==expected,'original input class pin differs')
        for tool in ('javac','java'):
            pin=run['tools'][jdk+'.'+tool]
            actual=file_pin(facts['home']/'bin'/tool)
            need(pin==actual and actual['sha256']==facts[tool],f'{jdk} {tool} live tool pin differs')
    need(len(run['tools'])==4,'tool pin inventory must be the two JDK javac/java pairs')
    need(len(run.get('source_variants',[]))==8 and len(run.get('runtime_comparisons',[]))==8,'expected eight source variants/comparisons')
    need(len(run.get('renders',[]))==4 and len(run.get('oracle_runs',[]))==2,'expected four renders and two fresh per-JDK oracles')
    # Exact CLI argv and render source/report binding, then full-method source-map equality.
    renders={(r['jdk'],r['evidence']):r for r in run['renders']}
    docs={}
    for key,render in renders.items():
        jdk,mode=key; idx=render['command_index']; command=commands[idx-100]
        need(idx==command['index'] and command['label']==f'jarde-render-{jdk}-{mode}','render command index does not bind row')
        input_path=workspace/JDKS[jdk]['class'][0]
        expected_argv=[str(cli_path), 'class-source','--input',str(input_path),'--class','cf07.LoopCases','--policy','single-class','--release','8','--format','json']
        if mode=='all': expected_argv += ['--evidence','all']
        need(command['argv']==expected_argv,f'{jdk}/{mode} CLI argv differs')
        doc=json.loads(stream_data[idx]['stdout'].decode('utf-8'))
        need(render['raw_json_stdout_sha256']==sha(stream_data[idx]['stdout']),'render raw JSON hash differs')
        source=Path(render['source']['path']); source_data=source.read_bytes()
        need(render['source']==file_pin(source),'render source output pin differs')
        need(source_data==doc['text'].encode('utf-8'),'saved source differs byte-for-byte from CLI class text')
        need(doc.get('class',{}).get('class_bytes',{}).get('length')==input_path.stat().st_size,'CLI class identity length differs from class input')
        docs[key]=doc
    summaries=[]
    for jdk in ('javac8','javac23'):
        default,all_doc=docs[(jdk,'default')],docs[(jdk,'all')]
        need(default['text']==all_doc['text'],'default/all full class text differs')
        need(len(default['methods'])==len(all_doc['methods']),'default/all method inventory differs')
        method_summaries=[]
        for a,b in zip(default['methods'],all_doc['methods']):
            key_a=(a['item']['name']['escaped'],bytes(a['item']['descriptor']['raw']).decode('ascii'))
            key_b=(b['item']['name']['escaped'],bytes(b['item']['descriptor']['raw']).decode('ascii'))
            need(key_a==key_b,'default/all method order/identity differs')
            ra=a.get('outcome',{}).get('report'); rb=b.get('outcome',{}).get('report')
            if isinstance(ra,dict) or isinstance(rb,dict):
                need(isinstance(ra,dict) and isinstance(rb,dict),'default/all recovered method report presence differs')
                need(ra['text']==rb['text'] and ra['source_map']==rb['source_map'],f'{jdk}/{key_a} method text/source map differs between evidence modes')
            need(a.get('text')==b.get('text'),'default/all method rendered text differs')
            method_summaries.append(key_a)
        summaries.append({'jdk':jdk,'method_count':len(method_summaries),'default_all_text_and_source_maps_equal':True})
    semantic=[]
    for jdk in ('javac8','javac23'):
        for mode in ('default','all'):
            doc=docs[(jdk,mode)]; class_path=workspace/JDKS[jdk]['class'][0]
            for method,desc in (('lastIndexOf','([IIII)I'),('counted','(II)I')):
                rep=report_for(doc,method,desc)
                summary=verify_loop_semantics(doc,class_path,method,rep,mode)
                summary.update({'jdk':jdk,'evidence':mode})
                semantic.append(summary)
    # Bind all ten runtime commands/JDK: each source variant must equal its fresh class-file oracle.
    comparisons={(x['jdk'],x['variant']):x for x in run['runtime_comparisons']}
    variants={(x['jdk'],x['variant']):x for x in run['source_variants']}
    runtime_checks=[]
    for jdk in ('javac8','javac23'):
        base=114 if jdk=='javac23' else 104
        oracle_compile,oracle_run=commands[base-100],commands[base-99]
        verify_compile_argv(oracle_compile,jdk,'oracle',run_dir)
        verify_java_argv(oracle_run,jdk,'oracle',run_dir)
        oracle_stdout=stream_data[oracle_run['index']]['stdout']; oracle_stderr=stream_data[oracle_run['index']]['stderr']
        need(oracle_stdout==b'0\n10\n39\n63\n9\n3\n-1\n3\n' and oracle_stderr==b'', 'original oracle differs from fixed eight source-derived rows')
        need(oracle_compile['exit_code']==0 and oracle_run['exit_code']==0,'original class oracle compile/run failed')
        for off,variant in enumerate(('original-source','jadx-frozen','jarde-default','jarde-all')):
            ci=base+2+2*off; ri=ci+1; compile_row=commands[ci-100]; runtime_row=commands[ri-100]
            verify_compile_argv(compile_row,jdk,variant,run_dir)
            verify_java_argv(runtime_row,jdk,variant,run_dir)
            result=variants[(jdk,variant)]; compare=comparisons[(jdk,variant)]
            need(result['matches_original_class_oracle'] is True and compare['equal'] is True and compare['stdout_equal'] is True and compare['stderr_equal'] is True,
                 f'{jdk}/{variant} execution record does not claim exact equality')
            out=stream_data[ri]['stdout']; err=stream_data[ri]['stderr']
            need(out==oracle_stdout and err==oracle_stderr,f'{jdk}/{variant} runtime streams differ from original class oracle')
            need(result['runtime_stdout_sha256']==sha(out) and result['runtime_stderr_sha256']==sha(err),'runtime result stream hash differs')
            classes=Path(compile_row['argv'][compile_row['argv'].index('-d')+1])
            inventory=sorted(p.relative_to(classes).as_posix() for p in classes.rglob('*.class'))
            need(inventory==['cf07/LoopCases.class','cf07/Runner.class'],f'{jdk}/{variant} compiled class inventory is not isolated/complete: {inventory}')
            for rel,recorded in result['generated_classes_post_compile_observed'].items():
                actual=file_pin(classes/'cf07'/rel)
                need(actual['bytes']==recorded['bytes'] and actual['sha256']==recorded['sha256'],'generated class hash differs from postcompile record')
            runtime_checks.append({'jdk':jdk,'variant':variant,'compile_index':ci,'runtime_index':ri,'stdout_equal':True,'stderr_equal':True,'complete_isolated_class_set':inventory})
        # The oracle runner directory must contain only Runner.class; raw class remains second on CP.
        oracle_dir=Path(oracle_compile['argv'][oracle_compile['argv'].index('-d')+1])
        need(sorted(p.relative_to(oracle_dir).as_posix() for p in oracle_dir.rglob('*.class'))==['cf07/Runner.class'],'oracle compile shadowed the original LoopCases class')
        original_dir=(workspace/JDKS[jdk]['class'][0]).parents[1]
        need(sorted(p.relative_to(original_dir).as_posix() for p in original_dir.rglob('*.class'))==['cf07/LoopCases.class'],'original class input directory changed or contains compiled shadows')
    # End-of-run pin attestations must match actual bytes again.
    need(run['end_pins_rechecked']['inputs']==len(run['inputs']) and run['end_pins_rechecked']['tools']==len(run['tools']),'replay end pin check counts differ')
    need(run['end_pins_rechecked']['runner']==run['runner'],'replay runner end pin differs')
    need(run['end_pins_rechecked']['build_receipt']==file_pin(build_path),'build receipt end pin differs')
    need(run['end_pins_rechecked']['cli']['sha256']==cli_sha and run['end_pins_rechecked']['cli']['bytes']==cli_path.stat().st_size,'CLI end pin differs')
    need(file_pin(RUNNER_PATH)['sha256']==RUNNER_SHA,'pinned replay runner changed during verification')
    return {'schema':'cf07-final-replay-independent-verification-luna-v1','status':'accepted','execution':{'path':str(execution_path),'bytes':len(raw_exec),'sha256':sha(raw_exec),'command_count':24,'raw_stream_count':48},
            'build':build_summary,'render_summaries':summaries,'render_method_semantics':semantic,
            'runtime_checks':runtime_checks,'fresh_jadx_execution_claimed':False,
            'runtime_side_count':10,'source_variant_count':8}


def verify_compile_argv(row,jdk,variant,run_dir):
    args=row['argv']; tool=JDKS[jdk]['home']/'bin/javac'
    need(Path(args[0])==tool,'javac executable is not pinned for selected JDK')
    flags=['-source','8','-target','8'] if jdk=='javac8' else ['--release','8']
    need(args[1:1+len(flags)]==flags,'javac release flags differ')
    need('-g:none' in args and '-proc:none' in args,'javac omitted pinned compile restrictions')
    classpath=Path(args[args.index('-classpath')+1]); sourcepath=Path(args[args.index('-sourcepath')+1])
    out=Path(args[args.index('-d')+1]); sources=[Path(x) for x in args[args.index('-d')+2:]]
    need(classpath.is_dir() and sourcepath.is_dir(),'javac class/source path does not exist')
    if variant=='oracle':
        need(classpath==workspace_class_dir(jdk) and sourcepath==out/'empty-sourcepath','oracle javac class/source path differs')
        need(sources==[ROOT/'openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/original-sources/Runner.java'],'oracle javac must compile Runner only')
    else:
        need(classpath==out.parent/'empty-classpath-sourcepath' and sourcepath==classpath,'variant javac must use empty isolated class/source path')
        need(len(sources)==2 and sources[0]==out.parent/'LoopCases.java' and sources[1]==out.parent/'Runner.java','variant javac source argv differs')
        need(sources[0].is_file() and sources[1].is_file(),'variant copied source/Runner missing')
        need(sources[1].read_bytes()==(ROOT/'openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/original-sources/Runner.java').read_bytes(),'variant Runner copy changed')
        if variant=='original-source': need(sources[0].read_bytes()==(ROOT/'openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/original-sources/LoopCases.java').read_bytes(),'original source copy changed')
        if variant=='jadx-frozen': need(sha(sources[0].read_bytes())==JADX_SOURCE_SHA,'JADX frozen source bytes changed')
        if variant.startswith('jarde-'):
            jdk_name='javac8' if jdk=='javac8' else 'javac23'; ev='default' if variant.endswith('default') else 'all'
            render_source=run_dir/'jarde-source'/jdk_name/ev/'LoopCases.java'
            need(sources[0].read_bytes()==render_source.read_bytes(),'Jarde source compile input differs from exact CLI text bytes')
    need(out.is_dir(),'javac destination directory missing')


def workspace_class_dir(jdk):
    return (ROOT/JDKS[jdk]['class'][0]).parents[1]


def verify_java_argv(row,jdk,variant,run_dir):
    args=row['argv']; need(Path(args[0])==JDKS[jdk]['home']/'bin/java','java executable is not pinned')
    need(args[1]=='-Xverify:all' and args[-1]=='cf07.Runner','runtime omitted full verification or Runner entry point')
    cp=args[args.index('-cp')+1].split(os.pathsep)
    if variant=='oracle':
        need(len(cp)==2 and Path(cp[0]).name=='runner-classes' and Path(cp[1])==workspace_class_dir(jdk),'oracle classpath must use isolated Runner then original class directory')
        need(not (Path(cp[0])/'cf07/LoopCases.class').exists(),'oracle Runner directory shadows original class')
        need((Path(cp[1])/'cf07/LoopCases.class').is_file(),'oracle original input class absent')
    else:
        compile_command=next(c for c in _CURRENT_RUN['commands'] if c['label']==f'compile-{variant}-{jdk}')
        classes=Path(compile_command['argv'][compile_command['argv'].index('-d')+1])
        need(cp==[str(classes)],'variant runtime classpath is not its isolated generated class directory')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--execution',required=True,type=Path)
    parser.add_argument('--build-receipt',required=True,type=Path)
    parser.add_argument('--cli',required=True,type=Path)
    parser.add_argument('--cli-sha256',required=True)
    parser.add_argument('--out',required=True,type=Path)
    args=parser.parse_args()
    global _CURRENT_RUN
    execution=args.execution.resolve(); build=args.build_receipt.resolve(); cli=args.cli.resolve(); out=args.out.resolve()
    need(out.parent.exists() and not out.exists(),'verification output must be a new private path')
    need(cli==CLI_PATH,'unexpected frozen CLI path')
    need(file_pin(RUNNER_PATH)['sha256']==RUNNER_SHA,'prepared replay runner pin mismatch')
    _CURRENT_RUN=json.loads(execution.read_text(encoding='utf-8'))
    result=verify_execution(execution,build,cli,args.cli_sha256,out,ROOT.resolve())
    result['verifier']=file_pin(Path(__file__).resolve())
    result['verifier_sha256_at_start']=result['verifier']['sha256']
    need(file_pin(Path(__file__).resolve())==result['verifier'],'verifier changed during run')
    out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':result['status'],'commands':24,'output':str(out)},indent=2))

if __name__=='__main__':
    main()
