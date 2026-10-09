from pathlib import Path
import hashlib,json,sys,zipfile
root=Path(__file__).resolve().parent
run=root/sys.argv[1]
d=json.loads((run/'manifest.json').read_text())
issues=[]; checked=0; bci_checks=0; site_count=0
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def check(ok,message):
 global checked
 checked+=1
 if not ok: issues.append(message)
for f in d['files']: check(sha(run/f['path'])==f['sha256'],f['path'])
for c in d['commands']:
 for stream in ['stdout','stderr']: check(sha(run/c[stream])==c[stream+'_sha256'],c['label']+' '+stream)
check(sha(Path(d['cli']))==d['cli_sha256'],'CLI')
expected={'Base','Mid','DirectA','DirectB','TwoHop','LocalInterface','Main'}
for c in d['cases']:
 check(c['accepted'] and c['compile_exit']==0,c['leg']+' accepted')
 check(len(c['generated_source_set'])==7 and {Path(p).stem for p in c['generated_source_set']}==expected,'source closed set')
 with zipfile.ZipFile(run/c['input_jar']) as z: check(set(z.namelist())=={n+'.class' for n in expected},'input closed set')
 commands=[x for x in d['commands'] if x['label'].startswith(c['leg']+'-jarde')]
 compile=next(x for x in commands if x['label'].endswith('-compile')); argv=compile['argv']
 for flag in ['-classpath','-sourcepath']:
  p=Path(argv[argv.index(flag)+1]); check(p.is_dir() and not list(p.iterdir()),'isolated '+flag)
 runtime=c['runtime']; argv=runtime['argv']; check('-Xverify:all' in argv,'verify all'); p=Path(argv[argv.index('-cp')+1]);check(p.name=='classes' and c['leg']+'-jarde' in str(p),'candidate only CP')
 for report in c['reports']:
  obj=json.loads((run/report['report_stdout']).read_text())
  source=next(p for p in c['generated_source_set'] if Path(p).stem==report['class'])
  check((run/source).read_text()==obj['text'],'all report source '+report['class'])
  check('@bytecode' not in obj['text'] and 'jarde_refused_body' not in obj['text'],'no refused body '+report['class'])
  if report['class']!='Main': continue
  for m in obj['methods']:
   name=next((n for n in ['sequence','collections','failures','ownDirect','ownTwoHop','ownInterface'] if n+'()' in m['declaration']),None)
   if name is None:continue
   r=m['outcome']['report']; news=r['news']; check(len(news)==2 and len({n['head'] for n in news})==2 and all(n['presented'] and n['refusal'] is None for n in news),name+' unique sites'); site_count+=len(news)
   bc=set()
   def walk(v):
    if isinstance(v,dict):
     if isinstance(v.get('bci'),int):bc.add(v['bci'])
     for x in v.values():walk(x)
    elif isinstance(v,list):
     for x in v:walk(x)
   for s in r['source_map']['segments']:walk(s['origin'])
   stores=[21,28,46,53] if name=='collections' else [18,33]
   target=set(stores)
   for n in news: target.update([n['head'],n['dup'],n['constructor']])
   for bci in target:check(bci in bc,name+' source bci '+str(bci));bci_checks+=1
out={'manifest_sha256':sha(run/'manifest.json'),'checks':checked,'site_count':site_count,'source_bci_checks':bci_checks,'issues':issues,'accepted_cases':sum(c['accepted'] for c in d['cases']),'whole_source_sets':True,'original_helpers_borrowed':False}
p=root/(sys.argv[1]+'-root-verification.json');p.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out));sys.exit(bool(issues))
