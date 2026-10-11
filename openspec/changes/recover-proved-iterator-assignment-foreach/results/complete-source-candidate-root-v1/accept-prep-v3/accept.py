#!/usr/bin/env python3
"""Verify the fixed-candidate and controls full-class runtime observations."""
from __future__ import annotations
import argparse, hashlib, importlib.util, json, os, re, stat, sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
BASES = {
    'fixed': Path('/private/tmp/jarde-iterator-assignment-fixed-candidate-root-v1'),
    'controls': Path('/private/tmp/jarde-iterator-assignment-controls-candidate-root-v1'),
}
RUNNERS = {
    'fixed': ('/private/tmp/jarde-iterator-assignment-fixed-candidate-root-v1.py', 'e3df2a88b6738200e94e03f5c816cb79200f9fd79cf1f36d6e0ef6adba8e5152'),
    'controls': ('/private/tmp/jarde-iterator-assignment-controls-candidate-root-v1.py', '9110399e5e2429fe8d8777435d137417f10b77862b6715b247bd68b61c43ff73'),
}
COLLECTOR = ('/private/tmp/jarde-for-condition-collector-root-v3/collect_controls.py', 'f944a892f4920feaff9631822bddf4dd9e49a0e8349ec4e09e4a16046257a7ac')
CHECKER = ('/private/tmp/jarde-for-condition-checker-root-v7/check_controls.py', 'a05acce687bf0e22bfab9eb8ae00520d53a006b740732f4b0041182ce72a464a')
CLOSURE = ('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/workspace_runner.py', '711fbd12ba12f8e559340f5dc8bcb3edd6a5dd6b075c9cf54677353565bc506e')
FOCUSED = ('/private/tmp/jarde-iterator-assignment-regressions-root-v1/execution.json', '0d6eb4e83f2b3de6aa250e1b9586b9f19c2f172ba782451b18149f0d2a552a2b')
COLLECTOR_ARG_ORDER = [
    'cli', 'cli_sha256', 'build_receipt', 'build_receipt_sha256',
    'receipt_cli_object', 'receipt_status_key', 'receipt_accepted_value', 'receipt_mode_key',
    'jdk_manifest', 'jdk_manifest_sha256', 'jadx', 'jadx_sha256', 'jadx_jar',
    'jadx_jar_sha256', 'guard', 'guard_sha256', 'scanner', 'scanner_sha256',
    'classfile_parser', 'classfile_parser_sha256', 'inputs_dir', 'inputs_manifest', 'out',
]
CLI = Path('/private/tmp/jarde-iterator-assignment-candidate-cli-root-v1')
CLI_SHA = '526e2e5a6db8a798a92d491123264bbb4d7105c0c1c3c2c9495c1b316eb89dd5'
BUILD = ('/private/tmp/jarde-iterator-assignment-regressions-root-v1/execution.json', FOCUSED[1])
MIN_FREE, MAX_TARGET = 5 * 1024**3, 1024**3
LABELS = ['javac8-original-compile','javac8-original-run','javac8-original-javap-{target}',
 'javac23-original-compile','javac23-original-run','javac23-original-javap-{target}',
 'jadx-version','jadx-default','javac8-jadx-default-compile','javac8-jadx-default-run',
 'javac23-jadx-default-compile','javac23-jadx-default-run','jadx-none','javac8-jadx-none-compile',
 'javac8-jadx-none-run','javac23-jadx-none-compile','javac23-jadx-none-run',
 'javac8-jarde-default-{target}-render','javac8-jarde-all-{target}-render',
 'javac23-jarde-default-{target}-render','javac23-jarde-all-{target}-render',
 'javac8-jarde-default-combined-compile','javac8-jarde-default-combined-run',
 'javac8-jarde-all-combined-compile','javac8-jarde-all-combined-run',
 'javac23-jarde-default-combined-compile','javac23-jarde-default-combined-run',
 'javac23-jarde-all-combined-compile','javac23-jarde-all-combined-run']

def need(ok, msg):
    if not ok: raise ValueError(msg)
def h(b): return hashlib.sha256(b).hexdigest()
def file_sha(p): return h(Path(p).read_bytes())
def pin(p):
    p=Path(p); b=p.read_bytes(); return {'path':str(p),'bytes':len(b),'sha256':h(b)}
def read_json(p): return json.loads(Path(p).read_text(encoding='utf-8'))
def verify_pin(d, what):
    p=Path(d['path']); need(p.is_file() and not p.is_symlink(),f'{what} missing or symlink: {p}')
    b=p.read_bytes(); need(len(b)==d['bytes'] and h(b)==d['sha256'],f'{what} pin mismatch: {p}')
    return b
def helper():
    p=Path(CLOSURE[0]); need(file_sha(p)==CLOSURE[1], 'source closure helper SHA changed')
    spec=importlib.util.spec_from_file_location('jarde_observation_closure',p)
    mod=importlib.util.module_from_spec(spec); spec.loader.exec_module(mod); return mod
def live_source_pins(closure):
    paths=set(closure.source_paths()) | set((ROOT/'examples').rglob('*.rs'))
    return closure.pin_sources(paths)
def source_block(text, method):
    m=re.search(r'\b'+re.escape(method)+r'\s*\([^)]*\)\s*\{',text)
    need(m is not None,f'generated source missing method {method}')
    depth=0
    for i in range(m.end()-1,len(text)):
        if text[i]=='{': depth+=1
        elif text[i]=='}':
            depth-=1
            if depth==0:return text[m.start():i+1]
    raise ValueError(f'unclosed method {method}')
def expected_command_argv(label, capture, target, class_name):
    c=Path(capture); inp=c/'inputs'; legs=c/'legs'
    if label=='jadx-version': return ['/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx','--version']
    if label in ('jadx-default','jadx-none'):
        mode='default' if label=='jadx-default' else 'none'
        args=['/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx','--no-res','--config','none','--threads-count','1']
        if mode=='none': args += ['--rename-flags','none']
        return args+['-d',str(c/'jadx'/mode),str(inp/'for-condition-controls.jar')]
    jdk='javac8' if label.startswith('javac8-') else 'javac23'
    home={'javac8':'/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home',
          'javac23':'/Users/lordcasser/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'}[jdk]
    if '-jarde-' in label and label.endswith('-render'):
        mode='all' if '-all-' in label else 'default'
        classfile=legs/f'{jdk}-original/classes/cf11/{target}.class'
        args=[str(CLI),'class-source','--input',str(classfile),'--class',class_name,'--policy','single-class','--release','8','--format','json']
        if mode=='all': args += ['--evidence','all']
        return args
    if '-javap-' in label:
        leg=label.split('-javap-',1)[0]
        return [home+'/bin/javap','-p','-c','-s','-v','-classpath',str(legs/f'{leg}/classes'),class_name]
    if label.endswith('-compile'):
        leg=label[:-len('-compile')]
        empty=str(legs/f'{leg}/empty-paths'); classes=str(legs/f'{leg}/classes')
        if jdk=='javac8': flags=['-source','8','-target','8','-Xlint:-options','-proc:none','-g']
        else: flags=['--release','8','-proc:none','-g']
        if leg.endswith('-original'): srcs=[inp/f'{target}.java',inp/'Runner.java']
        elif '-jadx-' in leg:
            mode='none' if '-none' in leg else 'default'; srcs=[inp/f'jadx-{mode}'/f'{target}.java',inp/'Runner.java']
        else:
            mode='all' if '-all-' in leg else 'default'
            srcs=[c/'jarde'/f'{jdk}-jarde-{mode}-{target}'/f'{target}.java',inp/'Runner.java']
        return [home+'/bin/javac',*flags,'-classpath',empty,'-sourcepath',empty,'-d',classes,*map(str,srcs)]
    if label.endswith('-run'):
        leg=label[:-len('-run')]
        return [home+'/bin/java','-Xverify:all','-cp',str(legs/f'{leg}/classes'),'cf11.Runner']
    raise ValueError(f'unknown command label for argv reconstruction: {label}')
def verify_launch(name, launch_path, expected_sha, closure, focused):
    raw=Path(launch_path).read_bytes(); need(h(raw)==expected_sha,f'{name} launch SHA mismatch')
    launch=json.loads(raw); base=BASES[name]
    need(launch.get('schema')=='iterator-assignment-candidate-launch-v1' and launch.get('status')=='checked_observations_only',f'{name} launch schema/status differs')
    rp,rs=RUNNERS[name]; need(launch.get('runner')=={'path':rp,'sha256':rs} and file_sha(rp)==rs,f'{name} runner pin/live SHA mismatch')
    need(launch.get('collector_exit_code')==0 and launch.get('checker_exit_code')==0,f'{name} collector/checker did not exit 0')
    need(launch.get('target_absent') is True and not (ROOT/'target').exists(),f'{name} final cargo target is present')
    need(launch.get('source_closure_helper')=={'path':CLOSURE[0],'sha256':CLOSURE[1]},f'{name} closure helper pin differs')
    before,after=launch.get('source_pins_before'),launch.get('source_pins_after')
    need(isinstance(before,dict) and isinstance(after,dict) and set(before)==set(after) and before==after and len(before)==4744,f'{name} launch source closure changed or wrong size')
    need(before==live_source_pins(closure),f'{name} current live source closure differs')
    cap_path=base/'capture/execution.json'; checker_path=base/'checker.json'
    cap_raw=cap_path.read_bytes(); cap=json.loads(cap_raw); check=json.loads(checker_path.read_text())
    need(cap.get('schema')=='cf11-target-full-source-capture-v2' and cap.get('status')=='observed',f'{name} capture schema/status differs')
    need(check.get('schema')=='cf11-target-full-source-observation-v2' and check.get('status')=='verified_observation_capture',f'{name} checker certificate status differs')
    need(check.get('execution')=={'path':str(cap_path),'bytes':len(cap_raw),'sha256':h(cap_raw)},f'{name} checker does not bind exact capture')
    need(check.get('actual_command_count')==29 and check.get('raw_stream_count')==58 and len(cap.get('commands',[]))==29,f'{name} command/raw inventory differs')
    target='UpstreamNestedLoops' if name=='fixed' else 'CF11OuterAssignmentTarget'
    class_name='cf11.'+target
    need(cap.get('package')=='cf11' and cap.get('classes')==[class_name] and cap.get('runner_class')=='cf11.Runner',f'{name} target identity differs')
    labels=[s.format(target=target) for s in LABELS]
    cmds=cap['commands']; need([r.get('label') for r in cmds]==labels,f'{name} command labels/order differ')
    need(cap.get('actual_command_count')==len(cmds),f'{name} actual command count inconsistent')
    need(cap.get('resource_guard')=={'minimum_free_bytes':MIN_FREE,'maximum_target_bytes':MAX_TARGET,'poll_seconds':1,'peak_target_sampled':True},f'{name} resource guard differs')
    config=cap.get('collector_config'); need(isinstance(config,dict),f'{name} collector config missing')
    need(config.get('cli')==str(CLI) and config.get('cli_sha256')==CLI_SHA and config.get('build_receipt')==BUILD[0] and config.get('build_receipt_sha256')==BUILD[1],f'{name} candidate CLI/build binding differs')
    need(Path(config['cli']).is_file() and not Path(config['cli']).is_symlink() and file_sha(config['cli'])==CLI_SHA and (Path(config['cli']).stat().st_mode & 0o777)==0o555,f'{name} candidate CLI live bytes/mode differs')
    need(cap.get('cli')==pin(CLI) and cap.get('cli_mode')=='0o555' and cap.get('build_receipt_cli_binding')=={'path':str(CLI),'bytes':CLI.stat().st_size,'sha256':CLI_SHA,'mode':'0o555'},f'{name} capture CLI receipt binding differs')
    need(config.get('build_receipt_sha256')==FOCUSED[1],f'{name} not tied to focused candidate receipt')
    need(launch.get('collector_config')==config and cap.get('collector_config')==config, f'{name} launch/capture collector configs differ')
    need(cap.get('source_tool_inputs_unchanged') is True and cap.get('source_tool_pins_before')==cap.get('source_tool_pins_after'),f'{name} source/tool inputs changed')
    st=cap['source_tool_pins_after']; need(len(st)==18,f'{name} tracked source/tool closure count differs')
    for row in st: verify_pin(row,f'{name} source/tool input')
    # Verify Python collector/checker invocation payloads exactly, including executable identity.
    collector=Path(COLLECTOR[0]); checker_src=Path(CHECKER[0])
    need(file_sha(collector)==COLLECTOR[1] and file_sha(checker_src)==CHECKER[1], 'collector/checker live source SHA changed')
    py=Path(launch['collector_argv'][0]).resolve()
    need(launch['collector_argv'][1]==str(collector) and Path(launch['collector_argv'][1]).resolve()==collector.resolve(),f'{name} collector argv script differs')
    need(launch['checker_argv'][1]==str(checker_src) and Path(launch['checker_argv'][1]).resolve()==checker_src.resolve(),f'{name} checker argv script differs')
    need(Path(launch['checker_argv'][0]).resolve()==py,f'{name} checker executable differs from collector Python')
    need(set(config)==set(COLLECTOR_ARG_ORDER) and len(COLLECTOR_ARG_ORDER)==len(config),f'{name} collector config key inventory differs')
    expected_collector_argv=[launch['collector_argv'][0],str(collector)]
    for key in COLLECTOR_ARG_ORDER: expected_collector_argv.extend(['--'+key.replace('_','-'),str(config[key])])
    need(launch['collector_argv']==expected_collector_argv,f'{name} full collector argv differs from receipt config')
    need(launch['collector_argv'][-1]==str(base/'capture') and launch['checker_argv'][-4:]==['--execution',str(cap_path),'--out',str(checker_path)],f'{name} launch argv outputs differ')
    need(config.get('out')==str(base/'capture'),f'{name} collector output path differs')
    # Every raw stream is independently re-read and hash checked; the expected 58-file set is exact.
    raw_dir=base/'capture/raw'; expected={f'{i}.{s}.raw' for i in range(1,30) for s in ('stdout','stderr')}
    need({p.name for p in raw_dir.iterdir()}==expected,f'{name} raw file inventory differs')
    by_label={}; stream_count=0; peak=0; min_free=None
    for i,row in enumerate(cmds,1):
        need(row.get('index')==i and row.get('cwd')==str(ROOT) and row.get('state')=='observed' and row.get('exit_code')==0 and row.get('guard_stop') is None,f'{name} command {i} status/cwd differs')
        argv=row.get('argv'); label=row['label']
        need(isinstance(argv,list) and argv and all(isinstance(x,str) for x in argv),f'{name} command {i} argv malformed')
        need(argv==expected_command_argv(label,base/'capture',target,class_name),f'{name} command {i} full argv differs from deterministic collector contract: {label}')
        if '-jarde-' in label and label.endswith('-render'):
            need(argv[0]==str(CLI),f'{name} command {i} Jarde CLI executable differs')
        elif label.startswith(('javac8-','javac23-')):
            jdk='javac8' if label.startswith('javac8-') else 'javac23'
            home={'javac8':'/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home',
                  'javac23':'/Users/lordcasser/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'}[jdk]
            tool='javac' if label.endswith('-compile') else ('javap' if '-javap-' in label else 'java')
            need(row.get('java_home')==home and argv[0]==home+'/bin/'+tool,f'{name} command {i} JDK tool/home differs')
        elif label.startswith('jadx-'):
            need(argv[0]==config.get('jadx') and row.get('java_home')=='/Users/lordcasser/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home',f'{name} command {i} JADX executable/home differs')
        else:
            raise ValueError(f'{name} command {i} has an unknown tool label: {label}')
        need(row.get('env_overrides')=={},f'{name} command {i} has env overrides')
        need(isinstance(row.get('peak_target_bytes'),int) and row['peak_target_bytes']<=MAX_TARGET and isinstance(row.get('free_bytes_after'),int) and row['free_bytes_after']>=MIN_FREE,f'{name} command {i} guard values outside bounds')
        peak=max(peak,row['peak_target_bytes']); min_free=row['free_bytes_after'] if min_free is None else min(min_free,row['free_bytes_after'])
        need(set(row.get('streams',{}))=={'stdout','stderr'},f'{name} command {i} missing streams')
        values={}
        for stream,ref in row['streams'].items():
            p=Path(ref['path']); need(p==raw_dir/f'{i}.{stream}.raw' and not p.is_symlink(),f'{name} command {i}/{stream} raw path differs')
            b=verify_pin(ref,f'{name} raw {i}/{stream}'); values[stream]=b; stream_count+=1
        by_label[row['label']]=values
    need(stream_count==58,f'{name} raw stream count differs')
    # Original JDK outputs are the oracle; all four JADX and four Jarde generated runtimes must match byte-for-byte.
    for jdk in ('javac8','javac23'):
        oracle=by_label[f'{jdk}-original-run']
        need(oracle['stdout'] and oracle['stderr']==b'',f'{name} {jdk} original oracle is empty or has stderr')
        for label,streams in by_label.items():
            if label.startswith(jdk+'-') and (label.endswith('-run') and ('jadx-' in label or '-combined-run' in label)):
                need(streams==oracle,f'{name} runtime differs from {jdk} original: {label}')
    need(by_label['javac8-original-run']==by_label['javac23-original-run'],f'{name} original JDK8/JDK23 runtime outputs differ')
    need(check.get('behavior_mismatches')==[] and check.get('behavioral_acceptance')=='not_claimed',f'{name} checker behavior observation fields differ')
    need(check.get('source_map_gaps')==[] and check.get('all_method_physical_coverage_complete') is True,f'{name} physical map coverage incomplete')
    need(check.get('jarde_unusable_outputs')==[] and check.get('compile_failures_remain_observations') is True,f'{name} checker compile observation inventory differs')
    need(check.get('default_all')==[{'jdk':'javac8','class':class_name,'both_rendered':True,'source_and_maps_equal':True},{'jdk':'javac23','class':class_name,'both_rendered':True,'source_and_maps_equal':True}],f'{name} default/all report agreement differs')
    profiles=cap.get('jarde_profiles'); need(isinstance(profiles,list) and len(profiles)==4,f'{name} must contain four Jarde source profiles')
    profile_keys={(r.get('jdk'),r.get('mode')) for r in profiles}; need(profile_keys=={('javac8','default'),('javac8','all'),('javac23','default'),('javac23','all')},f'{name} profile matrix differs')
    rendered=[]
    for row in profiles:
        need(row.get('render_exit')==0 and row.get('report_execution',{}).get('status')=='complete',f'{name} render/report incomplete')
        src=verify_pin(row['source_pin'],f'{name} generated source {row["label"]}').decode('utf-8')
        verify_pin(row['report_pin'],f'{name} generated report {row["label"]}')
        rendered.append(src)
    need(all(s==rendered[0] for s in rendered[1:]),f'{name} four Jarde source outputs are not byte-identical')
    # Derive per-method loop forms from source. Keep the pre-existing local5 empty declaration out of loop counts.
    methods=['test'] if name=='fixed' else ['positive','noCastAssign','bodyEffectBeforeAssign','extraNextConsumption']
    loop_rows=[]
    for method in methods:
        block=source_block(rendered[0],method)
        loops={'foreach':len(re.findall(r'\bfor\s*\(',block)),'while':len(re.findall(r'\bwhile\s*\(',block))}
        loop_rows.append({'method':method,**loops})
    if name=='fixed':
        need(loop_rows==[{'method':'test','foreach':2,'while':0}], 'fixed candidate source loop profile differs')
        need('java.lang.String s1;' in source_block(rendered[0],'test') and 's1 = (java.lang.String) iteratorElement' in source_block(rendered[0],'test'), 'fixed candidate outer String assignment/cast shape differs')
        need(check.get('jarde_method_maps_checked')==8, 'fixed method map count differs')
    else:
        need(loop_rows==[{'method':'positive','foreach':1,'while':0},{'method':'noCastAssign','foreach':0,'while':1},{'method':'bodyEffectBeforeAssign','foreach':0,'while':1},{'method':'extraNextConsumption','foreach':0,'while':1}], 'controls candidate per-method loop profile differs')
        need(check.get('jarde_method_maps_checked')==20, 'controls method map count differs')
    need(len(cap.get('jarde_compile_rows',[]))==4 and len(cap.get('jadx_compile_rows',[]))==4 and len(check.get('generated_compile_observations',[]))==8,f'{name} compile observation inventory differs')
    for row in cap['jarde_compile_rows']+cap['jadx_compile_rows']:
        need(row.get('compile_exit')==0 and row.get('run_exit')==0 and row.get('class_set_exact') is True and row.get('runtime_comparison',{}).get('same_jdk_original') is True,f'{name} generated compile/runtime row failed: {row.get("label")}')
    runtime_count=len([label for label in by_label if label.endswith('-original-run')])+len([label for label in by_label if label.endswith('-run') and ('jadx-' in label or '-combined-run' in label)])
    need(runtime_count==10, f'{name} runtime case count derived from command labels differs')
    return {'name':name,'launch_sha256':h(raw),'capture_sha256':h(cap_raw),'checker_sha256':file_sha(checker_path),
        'source_path_count':len(before),'all_live_source_pins_match':True,'commands':len(cmds),'raw_streams':stream_count,
        'peak_target_bytes':peak,'minimum_free_bytes_after_command':min_free,'source_profiles':len(profiles),
        'loop_rows':loop_rows,'runtime_observations':runtime_count,'runtime_equal_to_same_jdk_original':True,
        'jarde_method_maps_checked':check['jarde_method_maps_checked'],'default_all_equal':True,
        'behavior_mismatches':check['behavior_mismatches'],'behavioral_acceptance':check['behavioral_acceptance']}

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--focused-receipt',type=Path,required=True); p.add_argument('--focused-receipt-sha256',required=True)
    p.add_argument('--fixed-launch',type=Path,required=True); p.add_argument('--fixed-launch-sha256',required=True)
    p.add_argument('--controls-launch',type=Path,required=True); p.add_argument('--controls-launch-sha256',required=True)
    p.add_argument('--out',type=Path,required=True); a=p.parse_args()
    out=a.out.resolve(); need(out.parent.is_dir() and not out.exists(),'--out must be a new file in an existing directory')
    closure=helper(); fb=a.focused_receipt.read_bytes(); need(h(fb)==a.focused_receipt_sha256==FOCUSED[1], 'focused receipt pin differs')
    focused=json.loads(fb); need(focused.get('schema')=='iterator-assignment-regressions-root-v1' and focused.get('status')=='passed_candidate_build_only' and focused.get('target_absent') is True,'focused candidate receipt is not a successful focused-only run')
    need(focused.get('candidate_cli',{}).get('sha256')==CLI_SHA and pin(CLI)['sha256']==CLI_SHA and (CLI.stat().st_mode&0o777)==0o555,'focused candidate CLI bytes/mode differ')
    fixed=verify_launch('fixed',a.fixed_launch,a.fixed_launch_sha256,closure,focused)
    controls=verify_launch('controls',a.controls_launch,a.controls_launch_sha256,closure,focused)
    result={'schema':'iterator-assignment-two-class-runtime-observations-v1','status':'accepted_fixed_and_controls_runtime_observations_only',
      'scope':'Two captured source/class families and their named runtime cases only. This is not general behavioral equivalence, full-workspace acceptance, or a claim about unobserved inputs.',
      'focused_candidate_receipt':{'path':str(a.focused_receipt.resolve()),'sha256':h(fb)},
      'candidate_cli':{'path':str(CLI),'sha256':CLI_SHA,'mode':'0555'},'captures':[fixed,controls],
      'shared_source_closure_path_count':4744,'live_source_closure_matched_before_and_after':True,
      'both_final_targets_absent':not (ROOT/'target').exists()}
    result['acceptor']={'path':str(Path(__file__).resolve()),'sha256':file_sha(__file__)}
    out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':result['status'],'out':str(out),'captures':[x['capture_sha256'] for x in result['captures']]}))
    return 0
if __name__=='__main__':
    try: raise SystemExit(main())
    except Exception as e:
        print(f'observation acceptance failed: {e}',file=sys.stderr); raise SystemExit(1)
