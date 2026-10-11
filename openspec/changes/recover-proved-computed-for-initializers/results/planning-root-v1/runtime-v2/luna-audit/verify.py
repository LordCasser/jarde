#!/usr/bin/env python3
import hashlib, json, pathlib, sys
ROOT=pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
OUT=pathlib.Path('/private/tmp/jarde-cf07-cf13-current-runtime-root-v2')
PAT=pathlib.Path('/private/tmp/jarde-cf07-cf13-current-cli-patrol-root-v1')
MAN=ROOT/'openspec/changes/preserve-proved-if-arm-join-origins/results/cf07-candidate-root-v1/manifest.json'
def sha(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def check(ok,msg):
    if not ok: raise AssertionError(msg)
def j(p): return json.loads(pathlib.Path(p).read_text())
def main():
    ex=j(OUT/'execution.json'); patrol=j(PAT/'execution.json'); manifest=j(MAN)
    check(ex['status']=='passed_current_cli_two_cases_only','execution status')
    check(len(ex['commands'])==16,'expected 16 compile/runtime records')
    check(len(ex['results'])==4 and all(r['equal'] for r in ex['results']),'four paired results')
    check(len(patrol['commands'])==2,'patrol command count')
    check(patrol['product_commit']=='bafb61a5ec58b49f0040ea3a55420a56801bbe9c','patrol product commit')
    cli=patrol['commands'][0]['argv'][0]
    check(all(c['argv'][0]==cli for c in patrol['commands']),'patrol CLI path consistency')
    check(sha(cli)==patrol['cli_sha256'],'frozen CLI SHA')
    check(sha(cli)==ex['inputs'][0]['sha256'],'execution CLI pin')
    # Original report/source/class pins and exact structured facts.
    expected={
      'cf07': {'class':'cf07.LoopCases','method':'counted','class_sha':'004e99baeb062861f95a4dac13e3a78399a66efd31f2589b0028cfe50432ae33','stdout_sha':'6ba78b8e7c21c093513ab632e31e6a7e3917fa88a29841c749ef024d026a0348','primary_bci':14,'derived_bci':20,'start':271,'end':391,'slice':'        if (local3 == 7) {\n            local2 = local2 + 2;\n        } else {\n            local2 = local2 * 2;\n        }\n'},
      'cf13': {'class':'cf13.SwitchLoopExits','method':'walk','class_sha':'c5ae0cf54694e47fc025bb5268ea5220a321b40ed5e47cd21b6cecb93250833b','stdout_sha':'592def832056151ba92282361b324fd0ac95bf40b7bf57f878c540ecbcbda6f3'} }
    pins={x['path']:x['sha256'] for x in ex['inputs']}
    check(all(x['sha256']==pins[x['path']] for x in ex['inputs']),'conflicting duplicate input pin rows')
    for p,h in pins.items(): check(sha(p)==h,'execution input pin mismatch: '+p)
    bycase={'cf07':patrol['commands'][0], 'cf13':patrol['commands'][1]}
    for name,w in expected.items():
      pc=bycase[name]; argv=pc['argv']; inp=argv[argv.index('--input')+1]
      report=PAT/(name+'.stdout.raw'); source=PAT/(name+'.java')
      check(pc['exit_code']==0 and pc['input_sha256']==w['class_sha'],'patrol input/exit '+name)
      check(sha(inp)==w['class_sha']==pins[inp],'original class pin '+name)
      check(sha(report)==w['stdout_sha']==pc['stdout_sha256']==pins[str(report)],'report raw sha '+name)
      check(sha(source)==pins[str(source)],'reconstructed source pin '+name)
      text=source.read_text()
      doc=json.loads(report.read_text())
      check(text==doc.get('text'),'source exact report text '+name)
      check(argv[argv.index('--class')+1]==w['class'],'requested class '+name)
      methods=[m['outcome']['report'] for m in doc.get('methods',[]) if bytes(m.get('item',{}).get('identity',{}).get('name',[])).decode()==w['method']]
      check(len(methods)==1,'method row '+name)
      m=methods[0]
      check(m.get('quality')=='structured','structured method '+name)
      if name=='cf07':
        segments=m.get('source_map',{}).get('segments',[])
        targets=[s for s in segments if s.get('origin',{}).get('primary',{}).get('bci')==14]
        check(len(targets)==1,'CF07 source-map primary@14')
        seg=targets[0]
        check(seg['start']==w['start'] and seg['end']==w['end'],'CF07 exact segment span')
        check([x['bci'] for x in seg['origin'].get('derived',[])]==[20],'CF07 derived BCI20')
        check(m['text'].encode()[w['start']:w['end']].decode()==w['slice'],'CF07 exact complete If slice')
      else:
        check('continue;' in m.get('text','') and 'switch (' in m.get('text',''),'CF13 walk switch/continue')
    # Frozen JDK tool pins are repeated in both case groups; compare exact manifest legs.
    legs=manifest['jdk_legs']
    for leg in ('javac8','javac23'):
      for tool in ('javac','java','javap'):
        spec=legs[leg]['tools'][tool]
        check(sha(spec['path'])==spec['sha256'],'manifest JDK pin '+leg+'/'+tool)
        check(pins[spec['path']]==spec['sha256'],'execution JDK pin '+leg+'/'+tool)
    # Validate every actual stdout/stderr raw against its command record, plus argv/exit semantics.
    seen=set()
    for c in ex['commands']:
      label=c['label']; check(label not in seen,'duplicate command '+label); seen.add(label)
      check(c['exit_code']==0,'nonzero command '+label)
      for stream in ('stdout','stderr'):
        p=OUT/(label+'.'+stream+'.raw')
        h=sha(p)
        check(h==c[stream+'_sha256'],'raw stream SHA mismatch '+label+'/'+stream)
      a=c['argv']
      if label.endswith('-compile'):
        check(pathlib.Path(a[0]).name=='javac','compile executable '+label)
      else:
        check(pathlib.Path(a[0]).name=='java' and '-Xverify:all' in a,'runtime verify flag '+label)
    check(len(seen)==16,'all command labels')
    # Four exact paired byte streams; summaries must agree with those raw files.
    for case,profile in [('cf07','javac8'),('cf07','javac23'),('cf13','javac8'),('cf13','javac23')]:
      pair=[r for r in ex['results'] if r['case']==case and r['profile']==profile]
      check(len(pair)==1 and pair[0]['equal'],'result row '+case+'/'+profile)
      r=pair[0]
      check(r['stdout'].encode()==(OUT/(case+'-'+profile+'-original-runtime.stdout.raw')).read_bytes(),'original stdout summary/raw '+case+'/'+profile)
      check((OUT/(case+'-'+profile+'-original-runtime.stdout.raw')).read_bytes()==(OUT/(case+'-'+profile+'-jarde-runtime.stdout.raw')).read_bytes(),'pair raw equality '+case+'/'+profile)
      check(r['stdout'].encode()==(OUT/(case+'-'+profile+'-jarde-runtime.stdout.raw')).read_bytes(),'jarde stdout summary/raw '+case+'/'+profile)
      check((OUT/(case+'-'+profile+'-original-runtime.stderr.raw')).read_bytes()==b'','original runtime stderr '+case+'/'+profile)
      check((OUT/(case+'-'+profile+'-jarde-runtime.stderr.raw')).read_bytes()==b'','jarde runtime stderr '+case+'/'+profile)
      if case=='cf07': check(r['stdout']=='0\n10\n39\n63\n9\n3\n-1\n3\n','CF07 expected output')
      else: check(r['stdout']=='38\n','CF13 expected output')
    print('PASS: frozen CLI/report/input/JDK pins; CF07 exact span and BCI20; 16 raw compile/runtime records; four raw stdout-equivalent pairs')
if __name__=='__main__':
  try: main()
  except Exception as e:
    print('FAIL:',str(e),file=sys.stderr); raise
