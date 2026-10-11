from pathlib import Path
import hashlib,json
R=Path('/private/tmp/jarde-cf12-jdk8-switch-supplement-root-v4')
O=Path('/private/tmp/jarde-cf12-jdk8-assertions-overlay-root-v1')
def pin(p):
 p=Path(p);return {'path':str(p),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
def verify(x):assert pin(x['path'])==x,x['path']
o=json.loads((O/'execution.json').read_text());r=json.loads((R/'execution.json').read_text())
for x in o['pre_run_pins']+r['pre_run_pins']+o['overlay_classes']:verify(x)
assert o['overlay_usable'] and o['command']['exit_code']==0 and o['command']['guard_stop'] is None
for c in [o['command']]+r['commands']:
 for x in c['streams'].values():verify(x)
 assert c['guard_stop'] is None
cls=O/'classes';assert sorted(str(x.relative_to(cls)) for x in cls.rglob('*.class'))==['jadx/tests/api/utils/assertj/JadxAssertions.class']
b=next(cls.rglob('*.class')).read_bytes();assert b[:4]==b'\xca\xfe\xba\xbe' and int.from_bytes(b[6:8],'big')==52
assert r['actual_command_count']==2 and len(r['fixture_rows'])==1
f=r['fixture_rows'][0];assert f['fixture']=='TestSwitchFallThrough' and len(f['profiles'])==2
original,jadx=f['profiles'];assert original['side']=='original' and original['runtime']['exit_code']==0
actual=R/f['fixture']/'original/classes';expected={Path(x['path']).name:Path(x['path']).read_bytes() for x in f['inputs']}
assert {x.name:x.read_bytes() for x in actual.rglob('*.class')}==expected
assert jadx['side']=='jadx' and jadx['compile']['exit_code']==1 and jadx['runtime'] is None
err=Path(jadx['compile']['streams']['stderr']['path']).read_text()
assert 'ICodeInfo' in err and '55.0' in err and '52.0' in err
assert not list((R/f['fixture']/'jadx/classes').rglob('*.class'))
assert len(jadx['sources'])==1
for x in jadx['sources']:verify(x)
old=json.loads(Path(r['old_replay_execution']['path']).read_text())
a=next(x for x in old['fixtures'] if x['fixture']==f['fixture'])
j=next(x for x in a['jdk_profiles'] if x['jdk']=='jdk23_tools')
assert Path(original['runtime']['streams']['stdout']['path']).read_bytes()==Path(j['sides']['original']['runtime']['streams']['stdout']['path']).read_bytes()
assert r['cf12_complete'] is False
out={'schema':'cf12-jdk8-overlay-supplement-root-acceptance-v1','overlay_build':pin(O/'execution.json'),'supplement':pin(R/'execution.json'),'verifier':pin(__file__),'accepted_observation':True,'single_original_jdk8_runtime_passed':True,'generated_jdk8_environment_unavailable':'ICodeInfo major55 overload signature','complete_milestone':False,'cf12_complete':False,'commands':2,'target_absent':not Path('/Users/lordcasser/workspace/projects/jarde/target').exists()}
(R/'acceptance.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps(out,ensure_ascii=False))
