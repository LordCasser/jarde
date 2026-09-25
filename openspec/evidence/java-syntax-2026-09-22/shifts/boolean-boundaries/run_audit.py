from pathlib import Path
import subprocess,hashlib,json,shutil
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-shift-boolean-boundaries-0600');O=R/'openspec/evidence/java-syntax-2026-09-22/shifts/boolean-boundaries'
W.mkdir(exist_ok=True);O.mkdir(exist_ok=True);CLI=R/'target/debug/jarde-cli';sha=hashlib.sha256(CLI.read_bytes()).hexdigest()
def run(args,d,log):
 p=subprocess.run(args,text=True,capture_output=True,timeout=40);(d/log).write_text(p.stdout+p.stderr);return p.returncode
results=[]
for name,body,descriptor,calls in [
 ('ShiftBoolLeft','return value << distance;',b'(ZI)I','for(boolean v:new boolean[]{false,true})for(int d:new int[]{-33,-1,0,1,31,32,63})System.out.println(v+":"+d+":"+ShiftBoolLeft.shift(v,d));'),
 ('ShiftBoolDistance','return value >>> distance;',b'(IZ)I','for(int v:new int[]{Integer.MIN_VALUE,-1,0,1,Integer.MAX_VALUE})for(boolean d:new boolean[]{false,true})System.out.println(v+":"+d+":"+ShiftBoolDistance.shift(v,d));')]:
 d=O/name;d.mkdir(exist_ok=True);w=W/name;w.mkdir(exist_ok=True)
 source='public class '+name+' { public static int shift(int value,int distance){'+body+'} }\n';runner='public class '+name+'Runner {public static void main(String[] args){'+calls+'}}\n'
 (d/(name+'.java')).write_text(source);(d/(name+'Runner.java')).write_text(runner)
 assert run(['javac','--release','8','-g:none','-d',str(w/'original'),str(d/(name+'.java'))],d,'source-javac.log')==0
 k=w/'original'/(name+'.class');raw=k.read_bytes();needle=b'\x01\x00\x05(II)I';assert raw.count(needle)==1
 patched=raw.replace(needle,b'\x01\x00\x05'+descriptor);k.write_bytes(patched);shutil.copy2(k,d/(name+'.class'))
 assert run(['javac','--release','8','-cp',str(w/'original'),'-d',str(w/'original'),str(d/(name+'Runner.java'))],d,'runner-javac.log')==0
 assert run(['java','-Xverify:all','-cp',str(w/'original'),name+'Runner'],d,'original.txt')==0
 assert run(['javap','-v','-c','-p',str(k)],d,'javap.txt')==0
 p=subprocess.run([str(CLI),'class-source','--input',str(k),'--class',name,'--policy','single-class','--release','8','--format','text'],text=True,capture_output=True,timeout=40);assert p.returncode==0
 (d/'jarde.java.txt').write_text(p.stdout);(d/'jarde-report.txt').write_text(p.stderr);j=w/'jarde';j.mkdir(exist_ok=True);(j/(name+'.java')).write_text(p.stdout)
 s={'class':name,'bytes':len(patched),'class_sha256':hashlib.sha256(patched).hexdigest(),'patched_descriptor':descriptor.decode(),'cli_sha256':sha,'cases':len((d/'original.txt').read_text().splitlines()),'jarde_quotes':p.stdout.count('@bytecode')}
 s['jarde_javac']=run(['javac','--release','8','-d',str(j/'classes'),str(j/(name+'.java')),str(d/(name+'Runner.java'))],d,'jarde-javac.log')
 if s['jarde_javac']==0:
  s['jarde_run']=run(['java','-Xverify:all','-cp',str(j/'classes'),name+'Runner'],d,'jarde.txt');s['jarde_equal']=(d/'jarde.txt').read_bytes()==(d/'original.txt').read_bytes()
 s['jadx']=run(['jadx','--no-res','-d',str(w/'jadx'),str(k)],d,'jadx.log');assert s['jadx']==0
 g=next((w/'jadx').rglob(name+'.java'));t=g.read_text();(d/'jadx.java.txt').write_text(t);package=next((v for v in t.splitlines()if v.startswith('package ')),'');support=w/'support';support.mkdir(exist_ok=True);(support/(name+'Runner.java')).write_text(package+'\n'+runner)
 s['jadx_javac']=run(['javac','--release','8','-d',str(w/'jadx-classes'),str(g),str(support/(name+'Runner.java'))],d,'jadx-javac.log')
 if s['jadx_javac']==0:
  prefix=package[8:].rstrip(';')+'.'if package else ''
  s['jadx_run']=run(['java','-Xverify:all','-cp',str(w/'jadx-classes'),prefix+name+'Runner'],d,'jadx.txt');s['jadx_equal']=(d/'jadx.txt').read_bytes()==(d/'original.txt').read_bytes()
 results.append(s)
assert sha==hashlib.sha256(CLI.read_bytes()).hexdigest()
(O/'summary.json').write_text(json.dumps(results,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(results,indent=2))
