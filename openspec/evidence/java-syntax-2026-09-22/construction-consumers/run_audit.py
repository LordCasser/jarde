from pathlib import Path
import subprocess,json,hashlib
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-new-consumers');W.mkdir(exist_ok=True)
O=R/'openspec/evidence/java-syntax-2026-09-22/construction-consumers';O.mkdir(exist_ok=True)
sources={
'NewConsumers': 'public class NewConsumers {public static Runnable cast(){return (Runnable)(Object)new ConsumerValue();} public static void array(ConsumerValue[] a,int i){a[i]=new ConsumerValue();} public static boolean type(){return (Object)new ConsumerValue() instanceof Runnable;} public static int field(){return new ConsumerValue().number;} public static int call(){return new ConsumerValue().get();}}',
'ConsumerEffects': 'public class ConsumerEffects {public static int calls;public static boolean fail;public static final RuntimeException FAILURE=new IllegalStateException("constructor");}',
'ConsumerValue': 'public class ConsumerValue implements Runnable {public int number=7;public ConsumerValue(){ConsumerEffects.calls++;if(ConsumerEffects.fail)throw ConsumerEffects.FAILURE;} public void run(){}public int get(){return number;}}',
'ConsumerRunner': 'public class ConsumerRunner {interface Task{String run();}static void check(String name,boolean fail,Task task){ConsumerEffects.calls=0;ConsumerEffects.fail=fail;try{System.out.println(name+"="+task.run()+":"+ConsumerEffects.calls);}catch(RuntimeException e){System.out.println(name+"="+e.getClass().getName()+":"+(e==ConsumerEffects.FAILURE)+":"+ConsumerEffects.calls);}}public static void main(String[]args){for(boolean fail:new boolean[]{false,true}){check("cast"+fail,fail,()->""+(NewConsumers.cast() instanceof ConsumerValue));check("array"+fail,fail,()->{ConsumerValue[]a=new ConsumerValue[1];NewConsumers.array(a,0);return ""+a[0].number;});check("null"+fail,fail,()->{NewConsumers.array(null,0);return "returned";});check("bounds"+fail,fail,()->{NewConsumers.array(new ConsumerValue[0],0);return "returned";});check("type"+fail,fail,()->""+NewConsumers.type());check("field"+fail,fail,()->""+NewConsumers.field());check("call"+fail,fail,()->""+NewConsumers.call());}}}'
}
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/name).write_text(p.stdout+p.stderr);return p.returncode
for name,s in sources.items():(W/(name+'.java')).write_text(s+'\n');(O/(name+'.java')).write_text(s+'\n')
inputs=[str(W/(n+'.java')) for n in sources]
assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+inputs,'original-javac.log')==0
assert run(['java','-Xverify:all','-cp',str(W/'original'),'ConsumerRunner'],'original.txt')==0
klass=W/'original/NewConsumers.class';run(['javap','-c','-v',str(klass)],'javap.txt')
p=subprocess.run([str(R/'target/debug/jarde-cli'),'class-source','--input',str(klass),'--class','NewConsumers','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);d=W/'jarde';d.mkdir(exist_ok=True);(d/'NewConsumers.java').write_text(p.stdout)
summary={'cases':len((O/'original.txt').read_text().splitlines()),'cli_sha256':hashlib.sha256((R/'target/debug/jarde-cli').read_bytes()).hexdigest(),'jarde_quotes':p.stdout.count('@bytecode'),'jarde_javac':run(['javac','--release','8','-d',str(d/'classes'),str(d/'NewConsumers.java')]+inputs[1:],'jarde-javac.log')}
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(klass)],'jadx.log')==0
gen=next((W/'jadx').rglob('NewConsumers.java'));text=gen.read_text();(O/'jadx.java.txt').write_text(text);package=next((x for x in text.splitlines() if x.startswith('package ')),'')
support=W/'support';support.mkdir(exist_ok=True)
for name in list(sources)[1:]:(support/(name+'.java')).write_text(package+'\n'+sources[name]+'\n')
summary['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(gen)]+[str(support/(n+'.java')) for n in list(sources)[1:]],'jadx-javac.log')
if summary['jadx_javac']==0:
 prefix=package[len('package '):].rstrip(';')+'.' if package else ''
 assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'ConsumerRunner'],'jadx.txt')==0
 summary['jadx_equal']=(O/'original.txt').read_bytes()==(O/'jadx.txt').read_bytes();assert summary['jadx_equal']
(O/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(summary,indent=2))
