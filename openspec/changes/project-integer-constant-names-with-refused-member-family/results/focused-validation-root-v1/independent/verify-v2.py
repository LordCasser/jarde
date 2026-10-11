#!/usr/bin/env python3
"""Read-only verifier for the guarded CF12 focused build; never runs Cargo or Git."""
from __future__ import annotations
import argparse, hashlib, json, os, re, stat
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
EXPECTED_BASE='a4b0acf4b04c65360b6a8a8fe7e4a819a62a2db3'
EXPECTED_OUT='/private/tmp/jarde-cf12-constant-focused-root-v1'
EXPECTED_CLI='/private/tmp/jarde-cf12-constant-candidate-cli-root-v1'
EXPECTED_TEST='facade::integer_constant_name_tests::refused_root_without_child_keeps_same_class_integer_constant_names'
RUNNER=ROOT/'openspec/changes/project-integer-constant-names-with-refused-member-family/results/planning-root-v1/jarde-constant-validation-runner-luna-v1.py'
RUNNER_SHA='33113471b5936155df232280488fa5320ee1cef55e853afd07a0e840ebccb12d'
CONDITIONAL=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/run-validation-build-root-v2.py'
CONDITIONAL_SHA='0a09602cebea718296c1ac55140c6ef6c9d85503de5dead1910f18abc4999587'
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SIZE_SCAN=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SIZE_SCAN_SHA='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
HARNESS_EXEC=ROOT/'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/execution.json'
HARNESS_EXEC_SHA='5ffa56c72b2df664d21cb7e9afa30c819cf4a5071cb009c9839e3ae17724f281'
INVENTORY=ROOT/'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/observation-input-inventory-root-v1.json'
INVENTORY_SHA='d46f23c9f2bef614fee0d9f77efcfca561ebd5d986ad3b85d5ba2a70a609789e'
ENV_OVERRIDES={'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always','CARGO_PROFILE_TEST_STRIP':'symbols'}
STRIPPED=['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH']
MIN_FREE=5*1024**3
MAX_TARGET=1024**3
CI_LINTS=['too_many_arguments','cloned_ref_to_slice_refs','collapsible_if','type_complexity','len_zero','needless_option_as_deref','needless_borrow','useless_conversion','large_enum_variant','question_mark','comparison_to_empty','op_ref','manual_range_patterns','if_same_then_else','filter_map_bool_then','filter_next','unneeded_struct_pattern','redundant_guards','map_identity','redundant_slicing','unnecessary_get_then_check','unnecessary_unwrap','redundant_locals','replace_box','map_clone','unnecessary_mut_passed','single_element_loop','unnecessary_to_owned','needless_lifetimes']
PRODUCT_PATHS={'Cargo.toml','Cargo.lock','crates/jarde-cli/Cargo.toml','crates/jarde-cli/src/main.rs','crates/jarde-cli/src/task.rs','crates/jarde-cli/src/export.rs','src/facade.rs','src/class_source.rs','src/lib.rs','.github/workflows/ci.yml'}
TEST_PATHS={'tests/member_family_identity.rs','tests/class_source.rs','tests/p3_nested_annotation_source.rs','tests/inner_class_static_mixed_folding.rs','tests/member_class_static_folding.rs'}
INCLUDE_RE=re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
SUMMARY_RE=re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored')
TEST_LINE_RE=re.compile(r'(?m)^test (\S+) \.\.\. ok$')
IGNORED_LINE_RE=re.compile(r'(?m)^test (\S+) \.\.\. ignored(?: .*)?$')

def sha(b): return hashlib.sha256(b).hexdigest()
def file_sha(p): return sha(Path(p).read_bytes())
def need(ok,msg):
    if not ok: raise AssertionError(msg)
def jload(p): return json.loads(Path(p).read_text(encoding='utf-8'))
def pin(path,size,digest,label):
    b=Path(path).read_bytes(); need(len(b)==size,f'{label}: size {len(b)} != {size}'); need(sha(b)==digest,f'{label}: SHA-256 mismatch'); return b
def resolve_source(rel): return (ROOT/rel).resolve()
def literal_closure(seed_paths):
    found=set(); scanned=set(); pending=list(sorted(seed_paths))
    while pending:
        rel=pending.pop()
        if rel in scanned or not rel.endswith('.rs'): continue
        scanned.add(rel)
        text=(ROOT/rel).read_text(encoding='utf-8')
        for literal in INCLUDE_RE.findall(text):
            child=(ROOT/rel).parent.joinpath(literal).resolve()
            need(child.is_file() and child.is_relative_to(ROOT.resolve()),f'bad literal include {rel} -> {literal}')
            child_rel=child.relative_to(ROOT).as_posix()
            if child_rel not in found:
                found.add(child_rel); pending.append(child_rel)
    need(len(found)>0,'literal include closure unexpectedly empty')
    need(found.intersection({
      'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchLabels.test/input/TestSwitchLabels$TestCls.class',
      'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchLabels.test/input/TestSwitchLabels$TestCls$Inner.class'})=={
      'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchLabels.test/input/TestSwitchLabels$TestCls.class',
      'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchLabels.test/input/TestSwitchLabels$TestCls$Inner.class'},'both accepted frozen fixture bytes must be in the include closure')
    return found

def exact_commands(core):
    clippy=['cargo','clippy','--workspace','--all-targets','--all-features','--locked','--']
    for lint in CI_LINTS: clippy += ['-A','clippy::'+lint]
    clippy += ['-D','warnings']
    return [
      ['git','rev-parse','HEAD'],
      ['cargo','fmt','--all','--','--check'],
      clippy,
      ['cargo','test','-p','jarde','--lib','--locked'],
      ['cargo','test','-p','jarde','--lib','--locked','--',core,'--exact','--nocapture'],
      ['cargo','test','-p','jarde','--test','member_family_identity','--locked','--','--nocapture'],
      ['cargo','test','-p','jarde','--test','class_source','--locked','--','--nocapture'],
      ['cargo','test','-p','jarde','--test','p3_nested_annotation_source','--locked','--','--nocapture'],
      ['cargo','test','-p','jarde','--test','inner_class_static_mixed_folding','--locked','--','--nocapture'],
      ['cargo','test','-p','jarde','--test','member_class_static_folding','--locked','--','--nocapture'],
      ['cargo','build','-p','jarde-cli','--locked'],
      ['cargo','clean'],
    ]
def verify_test_output(index,stdout,core):
    text=stdout.decode('utf-8',errors='replace')
    summaries=[tuple(map(int,x)) for x in SUMMARY_RE.findall(text)]
    need(len(summaries)==1,f'command {index}: expected one successful Cargo test summary; got {summaries}')
    passed,failed,ignored=summaries[0]
    need(failed==0 and passed+ignored>0,f'command {index}: test summary not successful')
    named=TEST_LINE_RE.findall(text); ignored_names=IGNORED_LINE_RE.findall(text)
    need(len(named)==passed,f'command {index}: named ok lines {len(named)} != passed count {passed}')
    need(len(ignored_names)==ignored,f'command {index}: named ignored lines {len(ignored_names)} != ignored count {ignored}')
    if index==4:
        need(summaries==[(1,0,0)],f'new API test summary must be exactly 1 passed, 0 failed, 0 ignored: {summaries}')
        need(named==[core],f'exact test command did not emit its unique named pass line: {named}')
        need(len(re.findall(r'(?m)^\s*#\[test\]\s*\n\s*fn\s+'+re.escape(core.split('::')[-1])+r'\s*\(', (ROOT/'src/facade.rs').read_text()))==1,'selected new unit test function is not unique in src/facade.rs')
    return {'summary':list(summaries[0]),'named_passed':len(named),'named_ignored':len(ignored_names),'test_names':named if index in range(3,10) else None}

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--evidence',required=True,type=Path)
    ap.add_argument('--out',required=True,type=Path)
    a=ap.parse_args(); ev=a.evidence.resolve(); out=a.out.resolve()
    d=jload(ev)
    need(d.get('schema')=='project-integer-constant-names-with-refused-member-family-validation-root-v1','execution schema mismatch')
    need(d.get('source_base_expected')==EXPECTED_BASE,'source base does not match the required settled SHA')
    need(d.get('output_directory')==EXPECTED_OUT and str(ev.parent)==EXPECTED_OUT,'evidence directory mismatch')
    need(d.get('frozen_cli_destination')==EXPECTED_CLI,'frozen CLI destination mismatch')
    # A running/partial bundle is explicitly not an acceptance result.
    if d.get('status')!='validation-passed-cli-frozen-target-cleaned':
        print(json.dumps({'status':'not_accepted_incomplete','execution_status':d.get('status'),'commands_recorded':len(d.get('commands',[])),'failure':d.get('failure')},indent=2))
        return 2
    need(len(CI_LINTS)==29,'internal exact CI lint allowance count mismatch')
    need(file_sha(RUNNER)==RUNNER_SHA and d['runner']=={'path':str(RUNNER),'sha256':RUNNER_SHA},'validation runner pin mismatch')
    need(file_sha(CONDITIONAL)==CONDITIONAL_SHA,'conditional runner pin mismatch')
    need(file_sha(GUARD)==GUARD_SHA,'guard template pin mismatch')
    need(file_sha(SIZE_SCAN)==SIZE_SCAN_SHA,'size scanner pin mismatch')
    need(d['imported_conditional_runner']['path']==str(CONDITIONAL) and d['imported_conditional_runner']['sha256']==CONDITIONAL_SHA and d['imported_conditional_runner']['cli_lint_allowlist_entries']==29 and d['imported_conditional_runner']['load_guard_reused'],'imported CI lint/guard provenance mismatch')
    gd=d['guard']; need(gd['path']==str(GUARD) and gd['sha256']==GUARD_SHA,'recorded guard pin mismatch')
    need(gd['size_scanner']=={'path':str(SIZE_SCAN),'sha256':SIZE_SCAN_SHA,'mode':'single-stat race-tolerant'},'recorded size scanner mismatch')
    need(gd['minimum_free_bytes']==MIN_FREE and gd['maximum_target_bytes']==MAX_TARGET and gd['poll_seconds']==1 and gd['process_group_termination'] is True,'resource guard contract mismatch')
    fm=d['fixture_manifest']; need(fm['harness_execution']=={'path':str(HARNESS_EXEC),'sha256':HARNESS_EXEC_SHA},'harness execution pin mismatch')
    need(fm['observation_inventory']=={'path':str(INVENTORY),'sha256':INVENTORY_SHA},'inventory pin mismatch')
    pin(HARNESS_EXEC,Path(HARNESS_EXEC).stat().st_size,HARNESS_EXEC_SHA,'harness execution')
    inv_bytes=pin(INVENTORY,Path(INVENTORY).stat().st_size,INVENTORY_SHA,'observation inventory')
    inventory=jload(INVENTORY); need(inventory.get('schema')=='cf12-closed-input-inventory-root-v1','observation inventory schema mismatch')
    inv={x['path']:x for x in inventory.get('files',[])}
    for rel,row in fm['fixtures'].items():
        path=Path(row['path']); pin(path,row['bytes'],row['sha256'],'fixture '+rel)
        evidence_rel=rel.split('/cf12-upstream-java-root-v1/',1)[1]
        need(inv.get(evidence_rel)=={'path':evidence_rel,'bytes':row['bytes'],'sha256':row['sha256']},'fixture absent/mismatched from accepted inventory')
    need(len(fm['fixtures'])==2,'expected root and Inner fixture pins')
    before=d['source_pins_before']; after=d['source_pins_after']; need(after is not None and after==before,'source/include/fixture pins changed during validation')
    need(set(before['product_sources'])==PRODUCT_PATHS and set(before['test_sources'])==TEST_PATHS,'frozen source pin path closure differs from runner')
    includes=literal_closure(PRODUCT_PATHS|TEST_PATHS)
    need(set(before['src_facade_and_test_literal_include_closure'])==includes,'dynamic literal include closure differs from execution pins')
    for key,pins in before.items():
        if key in ('fixture_manifests','actual_frozen_fixtures'): continue
        for rel,digest in pins.items():
            pin(ROOT/rel,Path(ROOT/rel).stat().st_size,digest,f'{key} {rel}')
    need(before['fixture_manifests']=={'harness_execution':HARNESS_EXEC_SHA,'observation_inventory':INVENTORY_SHA},'source-pin fixture manifests mismatch')
    need(before['actual_frozen_fixtures']=={rel:row['sha256'] for rel,row in fm['fixtures'].items()},'source-pin fixture hashes mismatch')
    # The request freezes product implementation to facade only; the build runner must not mutate source pins.
    need(d['environment_overrides']==ENV_OVERRIDES and d['stripped_environment_keys']==STRIPPED,'environment override/stripping set mismatch')
    rows=d['commands']; need(len(rows)==12,'runner must record exactly 12 commands')
    core=EXPECTED_TEST
    expected=exact_commands(core)
    need([row['index'] for row in rows]==list(range(12)),'command indices are not contiguous')
    for i,(row,argv) in enumerate(zip(rows,expected)):
        need(row['argv']==argv,f'command {i} argv differs from exact rootrunner sequence: {row["argv"]!r}')
        need(row['cwd']==str(ROOT),f'command {i} cwd mismatch')
        need(row['exit_code']==0 and row['guard_stop'] is None,f'command {i} failed/tripped guard')
        need(row['env_overrides']==ENV_OVERRIDES,f'command {i} environment overrides differ')
        need(row['free_bytes_after']>=MIN_FREE,f'command {i} free-space floor violated')
        need(row['peak_target_bytes']<=MAX_TARGET,f'command {i} target-size cap violated')
        for stream in ('stdout','stderr'):
            ref=row['streams'][stream]; pin(ref['path'],ref['bytes'],ref['sha256'],f'command {i} {stream}')
        if i in range(3,10):
            stream=Path(row['streams']['stdout']['path']).read_bytes()
            need(row['test_summary_check']['ok'] is True,'rootrunner did not approve Cargo test summary')
            detail=verify_test_output(i,stream,core)
            need(row['test_summary_check']['actual_summaries']==[detail['summary']],'runner test summary differs from raw stdout')
            if i==4:
                need(row['test_summary_check']['exact_core_test_name']==core and row['test_summary_check']['exact_core_test_passed_line'] is True,'unique core test pass is not exact')
            cleanup=row.get('cleanup',{}).get('completed_test_binary',{})
            need(cleanup.get('action')=='delete-this-completed-test-executable','completed test executable cleanup record missing')
            testbin=Path(cleanup['path']); need(not testbin.exists(),f'completed test binary remains: {testbin}')
            need(cleanup['size_bytes']>0 and re.fullmatch(r'[0-9a-f]{64}',cleanup['sha256']) is not None,'deleted test binary identity record malformed')
        else:
            need(row.get('test_summary_check',{}).get('required') is False,'non-test command incorrectly marked as test')
    # Exact selected API test is the one newly added frozen-root acceptance test.
    test_source=(ROOT/'src/facade.rs').read_text(encoding='utf-8')
    need(test_source.count('fn refused_root_without_child_keeps_same_class_integer_constant_names()')==1,'selected root-only API test is not unique')
    # The supplied Git read-only command must report the settled base; verifier never calls Git.
    need(Path(rows[0]['streams']['stdout']['path']).read_text(encoding='ascii').strip()==EXPECTED_BASE,'recorded read-only HEAD does not equal supplied source base')
    # Freeze: built output identity equals preserved binary, mode 0555; Cargo clean is last and successful.
    need(rows[10]['argv']==['cargo','build','-p','jarde-cli','--locked'] and rows[11]['argv']==['cargo','clean'],'build/clean sequence mismatch')
    freeze=d.get('freeze'); need(isinstance(freeze,dict),'frozen CLI record missing')
    built=freeze['built_binary']; frozen=freeze['frozen_cli']
    need(built['path']==str(ROOT/'target/debug/jarde-cli'),'built CLI path mismatch')
    need(frozen['path']==EXPECTED_CLI and frozen['mode']=='0o555','frozen CLI path/mode mismatch')
    need(built['sha256']==frozen['sha256'] and built['size_bytes']==frozen['size_bytes'],'frozen CLI differs from successful cargo build output')
    cli=Path(frozen['path']); actual=pin(cli,frozen['size_bytes'],frozen['sha256'],'frozen candidate CLI')
    need(stat.S_IMODE(cli.stat().st_mode)==0o555 and os.access(cli,os.X_OK),'frozen CLI is not mode 0555 executable')
    need(freeze['source_base']==EXPECTED_BASE and freeze['product_source_pins']==before['product_sources'],'frozen CLI source provenance mismatch')
    frozen_json=jload(ev.parent/'frozen-cli.json'); need(frozen_json==freeze,'frozen-cli.json differs from execution freeze')
    # Cargo clean must be successful and leave target absent or within the pinned target bound.
    target=ROOT/'target'; target_bytes=sum(p.stat().st_size for p in target.rglob('*') if p.is_file()) if target.exists() else 0
    need(target_bytes==0,'cargo clean did not remove all target files')
    # Pre-existing fixture count is runtime-derived; report it without assuming a total include count.
    summary={'status':'accepted','source_base':EXPECTED_BASE,'core_test_name':core,'core_test_summary':[1,0,0],
      'commands':12,'whole_library_and_five_family_suites':[{'index':i,**verify_test_output(i,Path(rows[i]['streams']['stdout']['path']).read_bytes(),core)} for i in [3,5,6,7,8,9]],
      'clippy_ci_allowlist_entries':29,'guard':{'minimum_free_bytes':MIN_FREE,'maximum_target_bytes':MAX_TARGET,'max_observed_target_bytes':max(r['peak_target_bytes'] for r in rows),'minimum_observed_free_bytes':min(r['free_bytes_after'] for r in rows)},
      'source_pins_unchanged':True,'production_source_scope':'src/facade.rs (parent-defined scope; verifier confirms frozen source pin closure remained unchanged during build)','cargo_manifests_stable_during_validation':before['product_sources']['Cargo.toml']==after['product_sources']['Cargo.toml'] and before['product_sources']['Cargo.lock']==after['product_sources']['Cargo.lock'],
      'dynamic_include_closure_count':len(includes),'fixture_count':len(fm['fixtures']),'frozen_cli':{'path':str(cli),'bytes':len(actual),'sha256':sha(actual),'mode':'0o555'},
      'completed_test_executables_deleted':7,'cargo_clean_succeeded':True,'post_clean_target_bytes':target_bytes}
    out.parent.mkdir(parents=True,exist_ok=True); out.write_text(json.dumps(summary,indent=2,sort_keys=True)+'\n',encoding='utf-8')
    print(json.dumps(summary,indent=2,sort_keys=True))

if __name__=='__main__':
    try: main()
    except Exception as exc:
        print(f'NOT ACCEPTED: {type(exc).__name__}: {exc}')
        raise
