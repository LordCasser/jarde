from pathlib import Path
import subprocess,json
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
W=Path('/tmp/jarde-throw-grammar');W.mkdir(exist_ok=True)
O=ROOT/'openspec/evidence/java-syntax-2026-09-22/throws/grammar';O.mkdir(exist_ok=True)
sources={
'ThrowSwitch': 'public class ThrowSwitch {public static void select(int x,RuntimeException a,RuntimeException b){switch(x){case 0:throw a;case 1:throw b;default:throw null;}}}',
'ThrowClinit': 'public class ThrowClinit {static {if(true)throw new IllegalStateException("init");}}',
'ThrowClinitBranch': 'public class ThrowClinitBranch {static {if(System.nanoTime()==0L)throw new IllegalArgumentException("a");else if(true)throw new IllegalStateException("b");}}'
}
def run(args,path):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);path.write_text(p.stdout+p.stderr);return p.returncode
summary=[]
for name,source in sources.items():
 o=O/name;o.mkdir(exist_ok=True);w=W/name;w.mkdir(exist_ok=True)
 f=w/(name+'.java');f.write_text(source+'\n');(o/f.name).write_text(f.read_text())
 assert run(['javac','--release','8','-g:none','-d',str(w/'original'),str(f)],o/'original-javac.log')==0
 klass=w/'original'/(name+'.class');run(['javap','-v','-c',str(klass)],o/'javap.txt')
 p=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(klass),'--class',name,'--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
 (o/'jarde.java.txt').write_text(p.stdout);(o/'jarde-report.txt').write_text(p.stderr)
 rec=w/'jarde';rec.mkdir(exist_ok=True);(rec/f.name).write_text(p.stdout)
 item={'class':name,'quotes':p.stdout.count('@bytecode'),'jarde_javac':run(['javac','--release','8','-d',str(rec),str(rec/f.name)],o/'jarde-javac.log')}
 assert run(['jadx','--no-res','-d',str(w/'jadx'),str(klass)],o/'jadx.log')==0
 gen=next((w/'jadx').rglob(f.name));(o/'jadx.java.txt').write_text(gen.read_text())
 item['jadx_javac']=run(['javac','--release','8','-d',str(w/'jadx-classes'),str(gen)],o/'jadx-javac.log')
 if name.startswith('ThrowClinit'):
  runner='public class GrammarRunner {public static void main(String[] a){for(int i=0;i<2;i++){try{Class.forName(a[0]);System.out.println("loaded");}catch(Throwable e){Throwable c=e.getCause();System.out.println(e.getClass().getName()+":"+(c==null?"-":c.getClass().getName())+":"+(c==null?"-":c.getMessage().split(" ")[0]));}}}}'
  rf=w/'GrammarRunner.java';rf.write_text(runner+'\n');(o/'GrammarRunner.java').write_text(runner+'\n')
  assert run(['javac','--release','8','-d',str(w/'original'),str(rf)],o/'runner-javac.log')==0
  assert run(['java','-Xverify:all','-cp',str(w/'original'),'GrammarRunner',name],o/'original.txt')==0
  # A presentation hypothesis only; retain raw jarde text separately above.
  text=p.stdout;start=text.index('    static {')+len('    static {');end=text.rfind('    }')
  hypothetical=text[:start]+'\n        if (true) {'+text[start:end]+'        }\n'+text[end:]
  hyp=w/'hypothesis';hyp.mkdir(exist_ok=True);(hyp/f.name).write_text(hypothetical);(o/'hypothesis.java.txt').write_text(hypothetical)
  item['hypothesis_javac']=run(['javac','--release','8','-d',str(hyp),str(hyp/f.name),str(rf)],o/'hypothesis-javac.log')
  assert item['hypothesis_javac']==0
  assert run(['java','-Xverify:all','-cp',str(hyp),'GrammarRunner',name],o/'hypothesis.txt')==0
  item['hypothesis_equal']=(o/'original.txt').read_bytes()==(o/'hypothesis.txt').read_bytes();assert item['hypothesis_equal']
 summary.append(item)
(O/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(summary,indent=2))
