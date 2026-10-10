#!/usr/bin/env python3
"""Independently verify unused byte[] initializer baseline records; never runs toolchains."""
from __future__ import annotations
import hashlib, json, re
from pathlib import Path
from blake3 import blake3

ROOT = Path(__file__).resolve().parents[5]
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/unused-byte-array-init-next"
BASE = EVIDENCE / "baseline-root-v1"
RESULT = EVIDENCE / "results/unused-byte-array-init-independent-acceptance-v1.json"
MANIFEST_SHA = "787e4d29e5101913ce507175988d538f1e2651cdcd33da5899a9586004b98f4c"
INVENTORY_SHA = "4c4afae6929d55b74a36f89df8ae80944d113a4ca58e8f5b99187db3294ae650"
JDK_MANIFEST_SHA = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JARDE_PATH = Path('/private/tmp/jarde-instance-array-cli-v1')
JARDE_SHA = '5abb4bc82a50a8d427eff425ad80bc2fb845253b4216a81b60229e2337711663'
JADX_SHA = '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
SOURCE_SHA = '3c989a9563e4f36e71398cc387aa88b92539cfe35c1f4ab8f38b0a2de6df3909'
RUNNER_SHA = '822d4695ef96da2da97e20f626cd703740b010cce848f35071cdd6e33afb588f'
EMPTY_SHA = hashlib.sha256(b'').hexdigest()
METHODS = {'<init>()V', 'test()V'}
EXPECTED_FAILURES = [
    'javac8-original', 'javac23-original',
    "success counts differ: {'original': 0, 'jadx': 4, 'jarde': 4}",
]


def need(ok, msg):
    if not ok:
        raise AssertionError(msg)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_sha(path: Path) -> str:
    return sha(path.read_bytes())


def load(path: Path):
    return json.loads(path.read_text(encoding='utf-8'))


def bytes_row(root: Path, row: dict) -> bytes:
    p = root / row['path']
    data = p.read_bytes()
    need(len(data) == row['bytes'] and sha(data) == row['sha256'], f'file row mismatch: {p}')
    return data


def close_evidence():
    mp, ip = BASE/'manifest.json', BASE/'file-inventory.json'
    need(file_sha(mp) == MANIFEST_SHA, 'manifest SHA changed')
    need(file_sha(ip) == INVENTORY_SHA, 'inventory SHA changed')
    m, inv = load(mp), load(ip)
    need(m['schema'] == 'em18-unused-byte-array-init-baseline-luna-v1', 'unexpected collector schema')
    need(m['status'] == 'baseline-with-failures', 'unexpected collector status')
    rows = {r['path']: r for r in inv}
    need(len(rows) == len(inv) == 127, 'inventory duplicate/count mismatch')
    actual = {p.relative_to(BASE).as_posix() for p in BASE.rglob('*') if p.is_file() and p.name != 'file-inventory.json'}
    need(actual == set(rows), f'inventory not closed: missing={sorted(set(rows)-actual)} extra={sorted(actual-set(rows))}')
    for row in inv:
        bytes_row(BASE, row)
    need(m['file_inventory'] == {'path':'file-inventory.json','includes':['manifest.json','summary.json'],'excludes':['file-inventory.json']}, 'inventory policy differs')
    return m, rows


def frozen_tools(m):
    jr = m['jdk_manifest']; jp = Path(jr['path'])
    need(file_sha(jp) == JDK_MANIFEST_SHA == jr['sha256'], 'frozen JDK manifest mismatch')
    jm = load(jp); tr = jm['toolchain_manifest']; tp = Path(tr['path'])
    need(file_sha(tp) == tr['sha256'], 'toolchain manifest hash mismatch')
    tm = load(tp)
    need(tm['schema'] == 'bigdecimal-number-argument-original-v1', 'unexpected toolchain manifest')
    for leg, row in m['jdk_legs'].items():
        need(leg in ('javac8','javac23'), f'unknown JDK leg {leg}')
        pinned = next(x for x in tm['legs'] if x['leg'] == leg)
        for name in ('javac','java','javap'):
            a, b = row['tools'][name], pinned['tools'][name]
            need(a['path'] == b['path'] and a['sha256'] == b['sha256'] == file_sha(Path(b['path'])), f'{leg}/{name} tool pin mismatch')
    cli = m['frozen_jarde_cli']
    need(Path(cli['path']) == JARDE_PATH and cli['sha256'] == JARDE_SHA and file_sha(JARDE_PATH) == JARDE_SHA, 'Jarde binary pin mismatch')
    jx = m['jadx']
    need(jx['expected_version'] == '1.5.6' and jx['sha256'] == JADX_SHA and file_sha(Path(jx['resolved_launcher'])) == JADX_SHA, 'JADX pin mismatch')
    return jm


def verify_commands(m, rows):
    cmds = m['commands']; by = {}
    need(len(cmds) == 35, 'expected 35 commands')
    for c in cmds:
        label = c['label']; need(label not in by, f'duplicate command {label}'); by[label] = c
        need(c['exit'] == 0, f'command exit not zero: {label}')
        need(c['cwd'] == str(ROOT), f'command cwd mismatch: {label}')
        for sn in ('stdout','stderr'):
            raw = bytes_row(BASE, c['streams'][sn])
            need(c['streams'][sn]['path'] in rows and len(raw) == c['streams'][sn]['bytes'], f'untracked raw stream: {label}/{sn}')
    labels = {'jadx-version','jadx-default-decompile','jadx-none-decompile'}
    for leg in ('javac8','javac23'):
        labels |= {f'{leg}-{tool}-version' for tool in ('java','javac','javap')}
        labels |= {f'{leg}-original-{s}' for s in ('compile','run','javap')}
        for profile in ('default','none'):
            labels |= {f'{leg}-jadx-{profile}-{s}' for s in ('compile','run')}
        for profile in ('default','all'):
            labels |= {f'{leg}-jarde-render-{profile}',f'{leg}-jarde-{profile}-compile',f'{leg}-jarde-{profile}-run'}
    need(set(by) == labels, 'command labels not exactly closed')
    for leg in ('javac8','javac23'):
        for tool in ('java','javac','javap'):
            c = by[f'{leg}-{tool}-version']
            need(c['argv'] == [m['jdk_legs'][leg]['tools'][tool]['path'],'-version'], f'version argv mismatch: {leg}/{tool}')
            need(c['java_home'] == m['jdk_legs'][leg]['home'], f'version home mismatch: {leg}/{tool}')
    jx = m['jadx']; need(by['jadx-version']['argv'] == [jx['launcher'],'--version'], 'JADX version argv differs')
    for profile in ('default','none'):
        c = by[f'jadx-{profile}-decompile']; argv = [jx['launcher'],'--no-res','--config','none','--threads-count','1']
        if profile == 'none': argv += ['--rename-flags','none']
        argv += ['-d',str(BASE/f'jadx-output/{profile}'),str(BASE/'jadx-input/UnusedByteArrayInit.class.jar')]
        need(c['argv'] == argv, f'JADX argv mismatch: {profile}')
        record = next(x for x in jx['profiles'] if x['profile'] == profile)
        need(record['command'] == c and record['decompile_success'], f'JADX profile record mismatch: {profile}')
    return by


def parse_javap(text):
    lines=text.splitlines(); declarations=[]; bcis={}; current=None; descriptor=None; active=False
    open_index=next((i for i,line in enumerate(lines) if line.strip()=='{'),None)
    close_index=next((i for i in range(len(lines)-1,-1,-1) if lines[i].strip()=='}'),None)
    need(open_index is not None and close_index is not None and open_index<close_index,'javap class member block missing')
    direct_declarations=[line.strip() for line in lines[open_index+1:close_index]
                         if line.startswith('  ') and not line.startswith('    ') and line.strip().endswith(';')]
    need(direct_declarations==['public UnusedByteArrayInit();','public void test();'],
         f'javap physical declarations are not exactly two methods/no fields: {direct_declarations}')
    for line in lines:
        if line.startswith('  public UnusedByteArrayInit();'):
            current='<init>'; descriptor=None; active=False; declarations.append(line.strip())
        elif line.startswith('  public void test();'):
            current='test'; descriptor=None; active=False; declarations.append(line.strip())
        elif current and line.strip().startswith('descriptor:'):
            descriptor=line.split(':',1)[1].strip(); current += descriptor
        elif line.strip()=='Code:' and current:
            active=True; bcis[current]=set()
        elif active:
            mm=re.match(r'\s*(\d+):\s+',line)
            if mm: bcis[current].add(int(mm.group(1)))
            elif line.startswith('  public ') or line.strip()=='}': active=False
    need(declarations == ['public UnusedByteArrayInit();','public void test();'], f'javap member declarations differ: {declarations}')
    need(set(bcis)==METHODS, f'physical method identities differ: {set(bcis)}')
    need(len(bcis['<init>()V'])>0 and len(bcis['test()V'])>0, 'missing physical code')
    summaries=re.findall(r'\bfields:\s*(\d+),\s*methods:\s*(\d+),',text)
    need(len(summaries)<=1 and (not summaries or summaries[0]==('0','2')), 'javap member summary disagrees')
    return bcis


def verify_original_and_known_error(m, by):
    cases={c['label']:c for c in m['cases']}
    need(set(cases)=={'javac8-original','javac23-original'}|{f'{l}-jadx-{p}' for l in ('javac8','javac23') for p in ('default','none')}|{f'{l}-jarde-{p}' for l in ('javac8','javac23') for p in ('default','all')}, 'case labels differ')
    need(m['case_counts']=={'original':2,'jadx':4,'jarde':4} and m['expected_case_counts']=={'original':2,'jadx':4,'jarde':4}, 'case counts differ')
    need(m['success_counts']=={'original':0,'jadx':4,'jarde':4}, 'collector success count changed unexpectedly')
    need(m['failures']==EXPECTED_FAILURES, 'known collector failure set differs')
    facts={}
    for leg in ('javac8','javac23'):
        c=cases[f'{leg}-original']; need(c['kind']=='original' and c['jdk_leg']==leg, 'original case identity mismatch')
        need(c['compile_success'] and c['runtime_success'] and c['class_set_exact'] and c['complete_class_set'], f'actual original compile/runtime/class census failed: {leg}')
        need(c['success'] is False and c['javap']['physical_members_exact'] is False, f'expected recorded descriptor-assumption failure missing: {leg}')
        need(c['javap']['physical_fields']==[], f'original fields unexpectedly present: {leg}')
        jp=bytes_row(BASE,c['javap']['text']).decode('utf-8')
        bcis=parse_javap(jp)
        need(c['javap']['physical_methods']=={k:sorted(v) for k,v in bcis.items()}, f'collector BCI map disagrees with javap: {leg}')
        need(c['actual_class']['bytes']==187 and len(bytes_row(BASE,c['actual_class']))==187, f'original target class size differs: {leg}')
        run=by[c['runtime']['label']]
        need(run['exit']==0 and run['argv'][1:3]==['-Xverify:all','-cp'], f'original -Xverify runtime failed: {leg}')
        need(bytes_row(BASE,run['streams']['stdout'])==b'done\n' and bytes_row(BASE,run['streams']['stderr'])==b'', f'original Runner completion output differs: {leg}')
        facts[leg]={'methods':{k:sorted(v) for k,v in bcis.items()},'class_sha256':c['actual_class']['sha256'],'class_blake3':blake3(bytes_row(BASE,c['actual_class'])).hexdigest()}
    need(m['original_cross_jdk_raw_equal'] is True, 'cross-JDK original raw comparison absent')
    return cases,facts


def verify_report_and_initializer(m, by, facts):
    cases={c['label']:c for c in m['cases']}; out={}
    expected_stmt='byte[] local1 = new byte[]{10, 20, 30};'
    for leg in ('javac8','javac23'):
        original=cases[f'{leg}-original']; owner_bytes=bytes_row(BASE,original['actual_class']); owner_b3=blake3(owner_bytes).hexdigest()
        owner={'class_bytes':{'digest':owner_b3,'length':len(owner_bytes)},'location':{'kind':'standalone_root','snapshot':owner_b3},'variant':{'kind':'base'}}
        texts={}
        for mode in ('default','all'):
            label=f'{leg}-jarde-{mode}'; c=cases[label]; rp=c['rendered_profile']; cmd=rp['command']
            need(by[cmd['label']]==cmd, f'render command linkage differs: {label}')
            expected=[str(JARDE_PATH),'class-source','--input',str(BASE/f'cases/{leg}-original/classes/UnusedByteArrayInit.class'),'--class','UnusedByteArrayInit','--policy','single-class','--release','8','--format','json']
            if mode=='all': expected += ['--evidence','all']
            need(cmd['argv']==expected and cmd['java_home']==m['jdk_legs'][leg]['home'], f'Jarde argv differs: {label}')
            raw=bytes_row(BASE,cmd['streams']['stdout']); doc=json.loads(raw)
            need(doc['class']==owner and bytes_row(BASE,rp['document'])==raw, f'Jarde owner/document mismatch: {label}')
            text=doc['text'].encode('utf-8'); need(bytes_row(BASE,rp['generated_text'])==text, f'generated text differs from report: {label}')
            need(doc['outcome']=='performed' and doc['execution']['status']=='complete', f'report incomplete: {label}')
            need(doc['fields']==[] and len(doc['methods'])==2, f'physical member census mismatch: {label}')
            ids=set(); indexes=set(); method_reports={}
            for member in doc['methods']:
                item=member['item']; name=bytes(item['name']['raw']).decode('ascii'); desc=bytes(item['descriptor']['raw']).decode('ascii'); ident=name+desc
                need(ident in METHODS and ident not in ids and item['identity']['owner']==owner, f'method identity/owner mismatch: {label}/{ident}')
                ids.add(ident); indexes.add(item['index']); report=member['outcome']['report']
                need(member['outcome']['kind']=='recovered' and report['content']=='contains_statements' and report['fallbacks']==[], f'method not fully structured: {label}/{ident}')
                need(report['execution']['status']=='complete', f'method report incomplete: {label}/{ident}')
                cats={x['kind']:x['state']['state'] for x in report['evidence']['categories']}
                if mode=='all': need(cats.get('source_map')=='complete', f'all source map missing: {label}/{ident}')
                body=report['text'].encode('utf-8'); mapped=set()
                for seg in report['source_map']['segments']:
                    start,end=seg['start'],seg['end']; need(0<=start<end<=len(body), f'bad UTF-8 span: {label}/{ident}')
                    origin=seg['origin']; origins=([origin['primary']] if origin.get('primary') is not None else [])+origin.get('derived',[])
                    for o in origins:
                        need(o['method']['owner']==owner and o['method']['name']==item['name']['raw'] and o['method']['descriptor']==item['descriptor']['raw'], f'origin method mismatch: {label}/{ident}')
                        need(o['bci'] in facts[leg]['methods'][ident], f'origin BCI absent from javap: {label}/{ident}/{o["bci"]}')
                        mapped.add(o['bci'])
                need(mapped==facts[leg]['methods'][ident], f'source map does not cover method BCIs: {label}/{ident}')
                method_reports[ident]=report['text']
            need(ids==METHODS and indexes=={0,1}, f'method set/index mismatch: {label}')
            body=method_reports['test()V']
            need(body.count(expected_stmt)==1, f'local byte initializer missing/duplicated in test report: {label}')
            need(c['local_initializer_presentation']['method_identity']=='test()V' and c['local_initializer_presentation']['method_initializer_lines']==[expected_stmt], f'collector local fact differs: {label}')
            texts[mode]=text
        need(texts['default']==texts['all'], f'default/all Jarde source differs: {leg}')
        out[leg]={'class_sha256':sha(owner_bytes),'class_blake3':owner_b3,'method_bcis':facts[leg]['methods'],'default_all_text_equal':True,'initializer':expected_stmt}
    # Independently establish the two JADX source forms retain a local array initializer.
    for leg in ('javac8','javac23'):
        for profile in ('default','none'):
            c=cases[f'{leg}-jadx-{profile}']; p=BASE/c['generated_source']['path']; text=p.read_text(encoding='utf-8')
            need('byte[] bArr = {10, 20, 30};' in text, f'JADX local initializer absent: {leg}/{profile}')
    return out


def verify_class_legs(m, by):
    source=(EVIDENCE/'inputs-prepared-luna-v1/UnusedByteArrayInit.java').read_bytes(); runner=(EVIDENCE/'inputs-prepared-luna-v1/Runner.java').read_bytes()
    need(sha(source)==SOURCE_SHA and sha(runner)==RUNNER_SHA, 'prepared fixture source pin changed')
    need(m['prepared_input_sha256']=={'source':SOURCE_SHA,'runner':RUNNER_SHA}, 'manifest fixture source pins mismatch')
    for c in m['cases']:
        label=c['label']; leg=c['jdk_leg']; kind=c['kind']; need(c['compile_success'] and c['runtime_success'] and c['class_set_exact'] and c['complete_class_set'], f'compile/runtime/class set failed: {label}')
        root=BASE/'cases'/label; classdir=root/'classes'; empty=root/'empty-classpath-sourcepath'
        need(Path(c['class_output'])==classdir and Path(c['empty_classpath_sourcepath'])==empty and empty.is_dir() and not any(empty.iterdir()), f'compile isolation mismatch: {label}')
        expect={'Runner.class','UnusedByteArrayInit.class'}
        if kind=='jadx' and c['profile']=='default': expect={'defpackage/Runner.class','defpackage/UnusedByteArrayInit.class'}
        actual={p.relative_to(classdir).as_posix() for p in classdir.rglob('*.class')}
        need(set(c['expected_class_paths'])==expect and actual==expect==set(c['actual_class_paths']), f'compiled class set mismatch: {label}')
        src={r['path']:bytes_row(BASE,r) for r in c['source_files']}; t=f'cases/{label}/UnusedByteArrayInit.java'; r=f'cases/{label}/Runner.java'
        need(set(src)=={t,r}, f'compiled input set mismatch: {label}')
        if kind=='original': need(src[t]==source and src[r]==runner, f'original Java inputs differ: {label}')
        elif kind=='jadx':
            need(src[t]==bytes_row(BASE,c['generated_source']), f'JADX target changed: {label}')
            if c['profile']=='default': need(src[r]==b'package defpackage;\n\n'+runner, f'JADX Runner adaptation differs: {label}')
            else: need(src[r]==runner, f'JADX none Runner changed: {label}')
        elif kind=='jarde':
            doc=json.loads(bytes_row(BASE,c['rendered_profile']['document']))
            need(src[t]==doc['text'].encode() and src[r]==runner, f'Jarde inputs differ from report/original runner: {label}')
        else: raise AssertionError(f'unknown case kind: {kind}')
        cc=c['compile']; rc=c['runtime']; need(by[cc['label']]==cc and by[rc['label']]==rc, f'compile/runtime record not linked: {label}')
        need(cc['argv'][1:7]==['-source','8','-target','8','-g:none','-Xlint:-options'], f'javac flags differ: {label}')
        need(cc['argv'][0]==m['jdk_legs'][leg]['tools']['javac']['path'] and cc['java_home']==m['jdk_legs'][leg]['home'], f'javac pin mismatch: {label}')
        need(cc['argv'][7:11]==['-classpath',str(empty),'-sourcepath',str(empty)] and cc['argv'][11:13]==['-d',str(classdir)] and cc['argv'][13:]==[str(root/'UnusedByteArrayInit.java'),str(root/'Runner.java')], f'compile argv isolation/input mismatch: {label}')
        runner_class=('defpackage.Runner' if kind=='jadx' and c.get('profile')=='default' else 'Runner')
        need(rc['argv']==[m['jdk_legs'][leg]['tools']['java']['path'],'-Xverify:all','-cp',str(classdir),runner_class] and rc['java_home']==m['jdk_legs'][leg]['home'], f'run argv mismatch: {label}')
        class_rows={x['path']:x for x in c['classes']}; need(set(class_rows)=={f'cases/{label}/classes/{x}' for x in expect}, f'class output rows differ: {label}')
        for row in class_rows.values(): bytes_row(BASE,row)


def verify_oracles(m, by):
    originals={}; all_labels=[]
    for leg in ('javac8','javac23'):
        c=next(x for x in m['cases'] if x['label']==f'{leg}-original'); cmd=by[c['runtime']['label']]
        originals[leg]={n:bytes_row(BASE,cmd['streams'][n]) for n in ('stdout','stderr')}
        need(cmd['exit']==0 and originals[leg]['stderr']==b'', f'original raw execution failed: {leg}')
    need(originals['javac8']==originals['javac23'], 'original raw outputs differ by JDK')
    for c in m['cases']:
        cmd=by[c['runtime']['label']]; raw={n:bytes_row(BASE,cmd['streams'][n]) for n in ('stdout','stderr')}
        need(cmd['exit']==0 and cmd['argv'][1]=='-Xverify:all' and raw==originals[c['jdk_leg']], f'raw runtime differs from oracle: {c["label"]}')
        all_labels.append(c['label'])
    need(all_labels and len(all_labels)==10, 'expected ten full compile/run records')
    return {'cases':all_labels,'all_raw_match_same_jdk_original':True,'stdout_sha256':sha(originals['javac8']['stdout']),'stderr_sha256':sha(originals['javac8']['stderr']),'claim':'Runner completion only; it does not observe the local array or prove element values.'}


def main():
    result={'schema':'em18-unused-byte-array-init-independent-acceptance-luna-v1','success':False,'status':'not-verified','checks':[]}
    try:
        m,rows=close_evidence(); result['checks'].append('127-file evidence inventory closed and pinned')
        frozen_tools(m); by=verify_commands(m,rows); result['checks'].append('35 archived command records, raw streams, and tool pins')
        cases,facts=verify_original_and_known_error(m,by); result['checks'].append('original physical javap members independently confirm <init>()V/test()V; known stale descriptor assumption isolated')
        verify_class_legs(m,by); result['checks'].append('ten source/class-isolated compile records and exact class sets')
        result['jarde_reports']=verify_report_and_initializer(m,by,facts); result['checks'].append('four report owners, method/source-map origins, and local initializer source presentation')
        result['runtime']=verify_oracles(m,by); result['checks'].append('ten archived -Xverify:all raw runs match same-JDK original')
        result['collector_failures_preserved']=EXPECTED_FAILURES
        result['collector_original_success_flags']={k:cases[k]['success'] for k in ('javac8-original','javac23-original')}
        result['manifest_sha256']=MANIFEST_SHA; result['inventory_sha256']=INVENTORY_SHA; result['closed_file_count']=len(rows); result['command_count']=35; result['complete_compile_runtime_legs']=10
        result['status']='accepted-baseline-with-recorded-collector-error'; result['success']=True
    except Exception as exc:
        result['status']='verification-failed'; result['failure']=f'{type(exc).__name__}: {exc}'
    need(not RESULT.exists(), f'refusing to overwrite {RESULT}')
    RESULT.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(result,ensure_ascii=False,indent=2))
    return 0 if result['success'] else 1

if __name__=='__main__':
    raise SystemExit(main())
