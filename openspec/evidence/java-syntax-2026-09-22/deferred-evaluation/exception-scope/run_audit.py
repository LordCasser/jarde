from pathlib import Path
import subprocess,struct,hashlib,json,shutil
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-deferred-scope');O=R/'openspec/evidence/java-syntax-2026-09-22/deferred-evaluation/exception-scope';O.mkdir(exist_ok=True)
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/n).write_text(p.stdout+p.stderr);return p.returncode
names=['DeferredScope','ScopeSupport','ScopeRunner'];W.mkdir(exist_ok=True)
for n in names:
 if not (W/(n+'.java')).exists():shutil.copy2(O/(n+'.java'),W/(n+'.java'))
files=[str(W/(n+'.java')) for n in names]
for n in names:shutil.copy2(W/(n+'.java'),O/(n+'.java'))
assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,'source-javac.log')==0;k=W/'original/DeferredScope.class';assert run(['javap','-c','-v',str(k)],'source-javap.txt')==0
a=k.read_bytes()
def attr(c,start,end,handler):
 nested=struct.pack('>HIHB',22,6,1,64+handler)+b'\x07\x00\x0d'
 body=struct.pack('>HHI',1,1,len(c))+c+struct.pack('>HHHHHH',1,start,end,handler,13,1)+nested
 return struct.pack('>I',len(body))+body
before=attr(bytes.fromhex('b80007ac4b1007ac'),0,3,4);after=attr(bytes.fromhex('b80007b8000fac4b1007ac'),3,6,7);assert a.count(before)==1;a=a.replace(before,after);k.write_bytes(a)
assert run(['java','-Xverify:all','-cp',str(W/'original'),'ScopeRunner'],'original.txt')==0;assert run(['javap','-c','-v',str(k)],'patched-javap.txt')==0
p=subprocess.run([str(R/'target/debug/jarde-cli'),'class-source','--input',str(k),'--class','DeferredScope','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);assert p.returncode==0;(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);d=W/'jarde';d.mkdir(exist_ok=True);(d/'DeferredScope.java').write_text(p.stdout)
s={'class_sha256':hashlib.sha256(a).hexdigest(),'cases':3,'jarde_quotes':p.stdout.count('@bytecode'),'jarde_javac':run(['javac','--release','8','-d',str(d/'classes'),str(d/'DeferredScope.java')]+files[1:],'jarde-javac.log')}
if s['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(d/'classes'),'ScopeRunner'],'jarde.txt')==0;orig=(O/'original.txt').read_text().splitlines();got=(O/'jarde.txt').read_text().splitlines();assert len(orig)==len(got);s['jarde_differences']=sum(x!=y for x,y in zip(orig,got))
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0;g=next((W/'jadx').rglob('DeferredScope.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);package=next((x for x in t.splitlines() if x.startswith('package ')),'');sd=W/'support';sd.mkdir(exist_ok=True)
for n in names[1:]:(sd/(n+'.java')).write_text(package+'\n'+(W/(n+'.java')).read_text())
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g)]+[str(sd/(n+'.java')) for n in names[1:]],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.' if package else '';assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'ScopeRunner'],'jadx.txt')==0;s['jadx_equal']=(O/'original.txt').read_bytes()==(O/'jadx.txt').read_bytes()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
