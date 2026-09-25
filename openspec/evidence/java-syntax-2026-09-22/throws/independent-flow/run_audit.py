from pathlib import Path
import subprocess,json,sys
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-throw-flow-independent');WORK.mkdir(exist_ok=True)
EVIDENCE=ROOT/'openspec/evidence/java-syntax-2026-09-22/throws/independent-flow';EVIDENCE.mkdir(exist_ok=True)
stage=sys.argv[1] if len(sys.argv)>1 else 'before';assert stage in ['before','after']
OUT=EVIDENCE/stage;OUT.mkdir(exist_ok=True)
source='''public class ThrowFlow {
 public static void constructed(String text){throw new ThrowFlowException(ThrowFlowEffects.argument(text));}
 public static void nested(){throw ThrowFlowEffects.wrapper(ThrowFlowEffects.problem());}
 public static void array(RuntimeException[] errors,int index){throw errors[index];}
 public static void field(ThrowFlowHolder holder){throw holder.error;}
 public static void branch(boolean first,RuntimeException a,RuntimeException b){if(first)throw a;throw b;}
}'''
effects='''public class ThrowFlowEffects {
 public static int calls;public static int mode;
 public static final RuntimeException FAIL=new IllegalStateException("producer");
 public static final RuntimeException VALUE=new IllegalArgumentException("value");
 public static String argument(String text){calls=calls*10+1;if(mode==1)throw FAIL;return text;}
 public static RuntimeException problem(){calls=calls*10+3;if(mode==3)throw FAIL;return VALUE;}
 public static RuntimeException wrapper(RuntimeException x){calls=calls*10+4;if(mode==4)throw FAIL;return x;}
}'''
exception='''public class ThrowFlowException extends RuntimeException {
 public ThrowFlowException(String text){super(text);ThrowFlowEffects.calls=ThrowFlowEffects.calls*10+2;if(ThrowFlowEffects.mode==2)throw ThrowFlowEffects.FAIL;}
}'''
holder='''public class ThrowFlowHolder {public RuntimeException error;public ThrowFlowHolder(RuntimeException x){error=x;}}'''
runner='''public class ThrowFlowRunner {
 private static void attempt(String name,int mode,Runnable task){
  ThrowFlowEffects.calls=0;ThrowFlowEffects.mode=mode;
  try{task.run();System.out.println(name+"=returned");}
  catch(RuntimeException error){System.out.println(name+"="+error.getClass().getName()+":"+(error instanceof ThrowFlowException ? error.getMessage() : "-")+":"+(error==ThrowFlowEffects.FAIL)+":"+(error==ThrowFlowEffects.VALUE)+":"+ThrowFlowEffects.calls);}
 }
 public static void main(String[] args){
  attempt("constructed",0,()->ThrowFlow.constructed("message"));
  attempt("argumentFailure",1,()->ThrowFlow.constructed("message"));
  attempt("constructorFailure",2,()->ThrowFlow.constructed("message"));
  attempt("nested",0,()->ThrowFlow.nested());
  attempt("problemFailure",3,()->ThrowFlow.nested());
  attempt("wrapperFailure",4,()->ThrowFlow.nested());
  attempt("arrayValue",0,()->ThrowFlow.array(new RuntimeException[]{ThrowFlowEffects.VALUE},0));
  attempt("arrayElementNull",0,()->ThrowFlow.array(new RuntimeException[]{null},0));
  attempt("arrayNull",0,()->ThrowFlow.array(null,0));
  attempt("arrayBounds",0,()->ThrowFlow.array(new RuntimeException[0],0));
  attempt("fieldValue",0,()->ThrowFlow.field(new ThrowFlowHolder(ThrowFlowEffects.VALUE)));
  attempt("fieldNull",0,()->ThrowFlow.field(new ThrowFlowHolder(null)));
  attempt("receiverNull",0,()->ThrowFlow.field(null));
  attempt("first",0,()->ThrowFlow.branch(true,ThrowFlowEffects.VALUE,ThrowFlowEffects.FAIL));
  attempt("second",0,()->ThrowFlow.branch(false,ThrowFlowEffects.VALUE,ThrowFlowEffects.FAIL));
 }
}'''
sources={'ThrowFlow':source,'ThrowFlowEffects':effects,'ThrowFlowException':exception,'ThrowFlowHolder':holder,'ThrowFlowRunner':runner}
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(OUT/name).write_text(p.stdout+p.stderr);return p.returncode
for name,text in sources.items():(WORK/f'{name}.java').write_text(text+'\n');(EVIDENCE/f'{name}.java').write_text(text+'\n')
inputs=[str(WORK/f'{n}.java') for n in sources]
assert run(['javac','--release','8','-g:none','-d',str(WORK/'original')]+inputs,'original-javac.log')==0
assert run(['java','-Xverify:all','-cp',str(WORK/'original'),'ThrowFlowRunner'],'original.txt')==0
run(['javap','-v','-c',str(WORK/'original/ThrowFlow.class')],'javap.txt')
p=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(WORK/'original/ThrowFlow.class'),'--class','ThrowFlow','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
(OUT/'jarde.java.txt').write_text(p.stdout);(OUT/'jarde-report.txt').write_text(p.stderr)
rec=WORK/stage/'jarde';rec.mkdir(parents=True,exist_ok=True);(rec/'ThrowFlow.java').write_text(p.stdout)
summary={'stage':stage,'cases':len((OUT/'original.txt').read_text().splitlines()),'jarde_quotes':p.stdout.count('@bytecode')}
summary['jarde_javac']=run(['javac','--release','8','-d',str(rec/'classes'),str(rec/'ThrowFlow.java')]+inputs[1:],'jarde-javac.log')
if summary['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(rec/'classes'),'ThrowFlowRunner'],'jarde.txt')==0
 summary['jarde_equal']=(OUT/'original.txt').read_bytes()==(OUT/'jarde.txt').read_bytes()
 if stage=='after':assert summary['jarde_equal']
assert run(['jadx','--no-res','-d',str(WORK/stage/'jadx'),str(WORK/'original/ThrowFlow.class')],'jadx.log')==0
generated=next((WORK/stage/'jadx').rglob('ThrowFlow.java'));(OUT/'jadx.java.txt').write_text(generated.read_text())
package=next((x for x in generated.read_text().splitlines() if x.startswith('package ')),'')
support=WORK/stage/'support';support.mkdir(exist_ok=True)
for name in list(sources)[1:]:(support/f'{name}.java').write_text(package+'\n'+sources[name]+'\n')
summary['jadx_javac']=run(['javac','--release','8','-d',str(WORK/stage/'jadx-classes'),str(generated)]+[str(support/f'{n}.java') for n in list(sources)[1:]],'jadx-javac.log')
if summary['jadx_javac']==0:
 prefix=package[len('package '):].rstrip(';')+'.' if package else ''
 assert run(['java','-Xverify:all','-cp',str(WORK/stage/'jadx-classes'),prefix+'ThrowFlowRunner'],'jadx.txt')==0
 # Only the generated package prefix differs in the source-only exception class's printed name.
 observed=(OUT/'jadx.txt').read_text().replace(prefix+'ThrowFlowException','ThrowFlowException')
 assert observed==(OUT/'original.txt').read_text()
 summary['jadx_equal_after_package_normalization']=True
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');(EVIDENCE/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(summary,indent=2))
