from pathlib import Path
import subprocess, json
AUD=Path('/tmp/jarde-overload-boundaries/cases-v1')
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
CLI=ROOT/'target/debug/jarde-cli'
summary=json.loads((AUD/'summary.json').read_text())
for d in sorted(p for p in AUD.iterdir() if p.is_dir() and (p/'original-classes').exists()):
    name=d.name
    cmd=[str(CLI),'class-source','--input',str(d/f'{name}.class'),'--class',name,'--policy','single-class','--release','8','--format','text']
    p=subprocess.run(cmd,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    (d/'jarde.java.txt').write_text(p.stdout)
    (d/'jarde.report.txt').write_text('command: '+' '.join(cmd)+'\nexit: '+str(p.returncode)+'\n'+p.stderr)
    (d/'jarde-src'/f'{name}.java').write_text(p.stdout)
    # runner already present from first pass
    cp=subprocess.run(['javac','--release','8','-g:none','-d',str(d/'jarde-classes'),str(d/'jarde-src'/f'{name}.java'),str(d/'jarde-src'/'BoundaryRunner.java')],text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (d/'jarde-javac.log').write_text(cp.stdout)
    summary[name]['jarde_cli']=p.returncode==0
    summary[name]['jarde_compile']=cp.returncode==0
    if cp.returncode==0:
        rp=subprocess.run(['java','-cp',str(d/'jarde-classes'),'BoundaryRunner'],text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
        (d/'jarde-run.log').write_text(rp.stdout)
        summary[name]['jarde_run']=rp.returncode==0
    else:
        (d/'jarde-run.log').write_text('not run: jarde javac failed\n')
        summary[name]['jarde_run']=False
(AUD/'summary.json').write_text(json.dumps(summary,indent=2,ensure_ascii=False)+'\n')
print(json.dumps(summary,indent=2,ensure_ascii=False))
