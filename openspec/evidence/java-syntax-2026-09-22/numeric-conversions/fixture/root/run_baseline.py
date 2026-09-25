from pathlib import Path
import hashlib,json,shutil,subprocess
R=Path('/Users/lordcasser/workspace/projects/jarde');E=R/'openspec/evidence/java-syntax-2026-09-22'
def audit(name,fixture_dir,runner,support,work,evidence,patched):
 W=Path(work);O=E/evidence;O.mkdir(parents=True,exist_ok=True);W.mkdir(exist_ok=True)
 F=O/'inputs'
 def run(args,n):
  p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/n).write_text(p.stdout+p.stderr);return p.returncode
 source=[F/(v+'.java') for v in [name]+support+[runner]]
 original=W/'original';original.mkdir(exist_ok=True)
 if not patched:
  assert run(['javac','--release','8','-g:none','-d',str(original)]+list(map(str,source)),'source-javac.log')==0
 k=original/(name+'.class');f=F/'v8'/(name+'.class')
 assert k.read_bytes()==f.read_bytes()
 assert run(['javap','-c','-v','-p',str(k)],'javap.txt')==0
 assert run(['java','-Xverify:all','-cp',str(original),runner],'original.txt')==0
 a=(O/'original.txt').read_text().splitlines()
 s={'fixture_sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'bytes':f.stat().st_size,'code_methods':(O/'javap.txt').read_text().count('    Code:'),'source_or_patched_bytes_equal':True,'cases':len(a)}
 cli=R/'target/debug/jarde-cli';before=hashlib.sha256(cli.read_bytes()).hexdigest()
 p=subprocess.run([str(cli),'class-source','--input',str(k),'--class',name,'--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);assert p.returncode==0
 assert hashlib.sha256(cli.read_bytes()).hexdigest()==before
 s['jarde_cli_sha256']=before;s['jarde_quotes']=p.stdout.count('@bytecode');(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr)
 d=W/'jarde-root';d.mkdir(exist_ok=True);g=d/(name+'.java');g.write_text(p.stdout)
 s['jarde_javac']=run(['javac','--release','8','-d',str(d/'classes'),str(g)]+list(map(str,source[1:])),'jarde-javac.log')
 if s['jarde_javac']==0:
  assert run(['java','-Xverify:all','-cp',str(d/'classes'),runner],'jarde.txt')==0
  b=(O/'jarde.txt').read_text().splitlines();assert len(a)==len(b)
  s['jarde_differences']=[{'original':x,'jarde':y}for x,y in zip(a,b)if x!=y]
 assert run(['jadx','--no-res','-d',str(W/'jadx-root'),str(k)],'jadx.log')==0
 g=next((W/'jadx-root').rglob(name+'.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t)
 package=next((v for v in t.splitlines()if v.startswith('package ')),'');sup=W/'root-support';sup.mkdir(exist_ok=True)
 for src in source[1:]:(sup/src.name).write_text(package+'\n'+src.read_text())
 s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-root-classes'),str(g)]+[str(sup/src.name)for src in source[1:]],'jadx-javac.log')
 if s['jadx_javac']==0:
  prefix=package[8:].rstrip(';')+'.'if package else ''
  assert run(['java','-Xverify:all','-cp',str(W/'jadx-root-classes'),prefix+runner],'jadx.txt')==0
  s['jadx_equal']=(O/'jadx.txt').read_bytes()==(O/'original.txt').read_bytes()
 (O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_baseline.py').write_text(Path(__file__).read_text())
 print(json.dumps({k:v for k,v in s.items()if k!='jarde_differences'},indent=2));print('jarde_differences',len(s['jarde_differences']) if 'jarde_differences' in s else 'not-run: compilation failed')
audit('PrimitiveConversions','p3-primitive-conversions','PrimitiveConversionsRunner',['PrimitiveConversionSupport','PrimitiveConversionEffects'],'/var/folders/gm/lzn3ylzx4lz7mx29lftngdz40000gn/T/jarde-root-primitive-fixture-xy6ax_i2','numeric-conversions/fixture/root',False)
