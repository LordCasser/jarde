from pathlib import Path
import json,hashlib,importlib.util,os,stat
ROOT=Path('/Users/lordcasser/workspace/projects/jarde');HERE=Path(__file__).resolve().parent;OUT=HERE/'pop2-controls-root-v1'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
t=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py';assert sha(t)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
s=importlib.util.spec_from_file_location('pop2_control_guard',t);g=importlib.util.module_from_spec(s);s.loader.exec_module(g);OUT.mkdir(exist_ok=False);g.OUT=OUT;g.TEST_COMMANDS=set();g.EXPECTED_SUMMARIES={}
m=json.loads((HERE/'candidate-cli-v1.json').read_bytes());cls=HERE/'baseline-root-v1/javac23/original/discardprobe/ProbePop2.class';assert sha(cls)=='ea2d95ff242bba7ac66f7693b8a21dcca18e86d976be81728a6304d27e173e85'
env=os.environ.copy()
for k in g.STRIPPED_ENV_KEYS:env.pop(k,None)
env.update(g.ENV_VALUES)
rows=[];docs=[]
for cli,h in [(Path('/private/tmp/jarde-return-arm-latch-cli-v1'),'9ae5817d25749add0709b4f156c71ce23ac0e5ad41d66ee0136742024461a28f'),(Path(m['cli_path']),m['cli_sha256'])]:
 assert sha(cli)==h and stat.S_IMODE(cli.stat().st_mode)==0o555
 row=g.run_command(len(rows),[str(cli),'class-source','--input',str(cls),'--class','discardprobe.ProbePop2','--policy','single-class','--release','8','--format','json','--evidence','all'],env);rows.append(row)
 (OUT/'execution.json').write_text(json.dumps({'status':'running','commands':rows},indent=2)+'\n');assert row['exit_code']==0 and row['guard_stop'] is None
 p=Path(row['streams']['stdout']['path']);docs.append(json.loads((p if p.is_absolute() else ROOT/p).read_bytes()))
assert docs[0]['text']==docs[1]['text'] and docs[0]['class']==docs[1]['class']
assert len(docs[0]['methods'])==len(docs[1]['methods'])==3
for a,b in zip(docs[0]['methods'],docs[1]['methods']):
 assert a['item']==b['item'] and a['text']==b['text'] and a['outcome']['kind']==b['outcome']['kind']
 for key in ['text','source_map','quality','outcome','diagnostics','fallbacks']:assert a['outcome']['report'][key]==b['outcome']['report'][key],(a['item']['name'],key)
discard=next(x for x in docs[1]['methods'] if x['item']['name']['escaped']=='discardWide');r=discard['outcome']['report'];assert '@bytecode' in r['text'];assert all(x['bci']!=3 for seg in r['source_map']['segments'] for x in seg['origin']['derived'])
record={'schema':'discarded-call-pop2-control-root-acceptance-v1','status':'accepted-unchanged-control','commands':rows,'class_sha256':sha(cls),'all_three_methods_body_map_refusals_identical':True,'discard_quality':r['quality'],'diagnostics':r['diagnostics'],'no_new_derived_pop2':True};(OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
