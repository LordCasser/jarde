from pathlib import Path
import subprocess,json,shutil
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
BASE=Path('/tmp/jarde-floating-constants')
WORK=Path('/tmp/jarde-floating-nan-payload');WORK.mkdir(exist_ok=True)
EVIDENCE=ROOT/'openspec/evidence/java-syntax-2026-09-22/floating-constants/nan-payloads';EVIDENCE.mkdir(exist_ok=True)
raw=(BASE/'original/FloatingConstants.class').read_bytes()
fp=bytes.fromhex('047fc00000');dp=bytes.fromhex('067ff8000000000000');assert raw.count(fp)==raw.count(dp)==1
variants=[('positive-quiet',0x7fc12345,0x7ff8123456789abc),('negative-quiet',0xffc12345,0xfff8123456789abc),('positive-signaling',0x7f812345,0x7ff0123456789abc)]
summary=[]
for name,fbits,dbits in variants:
 w=WORK/name;w.mkdir(exist_ok=True);out=EVIDENCE/name;out.mkdir(exist_ok=True)
 def run(args,log):
  p=subprocess.run(args,capture_output=True,text=True,timeout=30);(out/log).write_text(p.stdout+p.stderr);return p.returncode
 patched=raw.replace(fp,b'\x04'+fbits.to_bytes(4,'big')).replace(dp,b'\x06'+dbits.to_bytes(8,'big'))
 (w/'FloatingConstants.class').write_bytes(patched)
 for helper in ['FloatingSupport','FloatingRunner']:shutil.copy2(BASE/'original'/f'{helper}.class',w/f'{helper}.class')
 assert run(['java','-Xverify:all','-cp',str(w),'FloatingRunner'],'original.txt')==0
 run(['javap','-v','-c',str(w/'FloatingConstants.class')],'javap.txt')
 assert run(['jadx','--no-res','-d',str(w/'jadx'),str(w/'FloatingConstants.class')],'jadx.log')==0
 generated=next((w/'jadx').rglob('FloatingConstants.java'));(out/'jadx.java.txt').write_text(generated.read_text())
 package=next((x for x in generated.read_text().splitlines() if x.startswith('package ')),'')
 support=w/'support';support.mkdir(exist_ok=True)
 for helper in ['FloatingSupport','FloatingRunner']:(support/f'{helper}.java').write_text(package+'\n'+(BASE/f'{helper}.java').read_text())
 assert run(['javac','--release','8','-d',str(w/'jadx-classes'),str(generated)]+[str(support/f'{n}.java') for n in ['FloatingSupport','FloatingRunner']],'jadx-javac.log')==0
 prefix=package[len('package '):].rstrip(';')+'.' if package else ''
 assert run(['java','-Xverify:all','-cp',str(w/'jadx-classes'),prefix+'FloatingRunner'],'jadx.txt')==0
 original=(out/'original.txt').read_text().splitlines();jadx=(out/'jadx.txt').read_text().splitlines();assert len(original)==len(jadx)
 diffs=[f'original {a}\njadx {b}' for a,b in zip(original,jadx) if a!=b];(out/'differences.txt').write_text('\n'.join(diffs)+'\n')
 p=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(w/'FloatingConstants.class'),'--class','FloatingConstants','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
 (out/'jarde.java.txt').write_text(p.stdout);(out/'jarde-report.txt').write_text(p.stderr)
 summary.append({'variant':name,'float_bits':f'{fbits:08x}','double_bits':f'{dbits:016x}','original_nan_lines':[x for x in original if 'Nan=' in x],'jadx_mismatches':len(diffs)})
(EVIDENCE/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');(EVIDENCE/'patch.py').write_text(Path(__file__).read_text());print(json.dumps(summary,indent=2))
