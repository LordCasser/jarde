from pathlib import Path
import subprocess,struct,hashlib,json,shutil
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/var/folders/gm/lzn3ylzx4lz7mx29lftngdz40000gn/T/jarde-root-deferred-accept-uhefu1l1/structured');O=R/'openspec/evidence/java-syntax-2026-09-22/deferred-evaluation/root-acceptance/structured';O.mkdir(exist_ok=True);W.mkdir(exist_ok=True)
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/n).write_text(p.stdout+p.stderr);return p.returncode
names=['DeferredStructured','StructuredSupport','StructuredRunner']
for n in names:
 if not (W/(n+'.java')).exists():shutil.copy2(O/(n+'.java'),W/(n+'.java'))
 shutil.copy2(W/(n+'.java'),O/(n+'.java'))
files=[str(W/(n+'.java')) for n in names];assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,'source-javac.log')==0;k=W/'original/DeferredStructured.class';assert run(['javap','-c','-v',str(k)],'source-javap.txt')==0;a=k.read_bytes();patches=[]
def attr(code,locals,frame):
 nested=struct.pack('>HIHB',25,3,1,frame)
 body=struct.pack('>HHI',1,locals,len(code))+code+struct.pack('>HH',0,1)+nested
 return struct.pack('>I',len(body))+body
for n,before,after,locals,oldframe,newframe in [('branch','1a990007b80007ac06ac','1a99000ab80007b80011ac06ac',1,8,11),('prefix','b8000d9900061007ac1009ac','b8000db800119900061007ac1009ac',0,9,12)]:
 old=attr(bytes.fromhex(before),locals,oldframe);assert a.count(old)==1;a=a.replace(old,attr(bytes.fromhex(after),locals,newframe));patches.append({'name':n,'before':before,'after':after})
k.write_bytes(a);assert run(['java','-Xverify:all','-cp',str(W/'original'),'StructuredRunner'],'original.txt')==0;assert run(['javap','-c','-v',str(k)],'patched-javap.txt')==0
p=subprocess.run([str(R/'target/debug/jarde-cli'),'class-source','--input',str(k),'--class','DeferredStructured','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);assert p.returncode==0;(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);d=W/'jarde';d.mkdir(exist_ok=True);(d/'DeferredStructured.java').write_text(p.stdout)
s={'class_sha256':hashlib.sha256(a).hexdigest(),'patches':patches,'cases':12,'jarde_quotes':p.stdout.count('@bytecode'),'jarde_javac':run(['javac','--release','8','-d',str(d/'classes'),str(d/'DeferredStructured.java')]+files[1:],'jarde-javac.log')}
if s['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(d/'classes'),'StructuredRunner'],'jarde.txt')==0;orig=(O/'original.txt').read_text().splitlines();got=(O/'jarde.txt').read_text().splitlines();assert len(orig)==len(got);s['jarde_differences']=sum(x!=y for x,y in zip(orig,got))
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0;g=next((W/'jadx').rglob('DeferredStructured.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);package=next((x for x in t.splitlines() if x.startswith('package ')),'');sd=W/'support';sd.mkdir(exist_ok=True)
for n in names[1:]:(sd/(n+'.java')).write_text(package+'\n'+(W/(n+'.java')).read_text())
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g)]+[str(sd/(n+'.java')) for n in names[1:]],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.' if package else '';assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'StructuredRunner'],'jadx.txt')==0;s['jadx_equal']=(O/'jadx.txt').read_bytes()==(O/'original.txt').read_bytes()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
