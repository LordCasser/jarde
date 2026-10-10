import ast,datetime,hashlib,json,os,shutil,signal,subprocess,time
from pathlib import Path
ROOT=Path('/Users/lordcasser/workspace/projects/jarde'); OUT=Path('/private/tmp/jarde-return-latch-boundary-compile-root-v1');OUT.mkdir(exist_ok=False)
JDK=Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'); CLI=Path('/private/tmp/jarde-proved-if-join-cli-v1')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(CLI)=='7b751758ee7f0b49bd61332a1a78412b36ecef898e43171ecf6af620d65cc9a6'
env=os.environ.copy()
for k in ('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'):env.pop(k,None)
guard=Path('/private/tmp/jarde-cf12-render-baseline-root-v1.py')
selected=[n for n in ast.parse(guard.read_text()).body if isinstance(n,ast.FunctionDef) and n.name in ('size','run')]
assert len(selected)==2;exec(compile(ast.Module(body=selected,type_ignores=[]),str(guard),'exec'))
record={'schema':'return-latch-real-boundary-input-root-v1','runner_sha256':sha(Path(__file__)),'guard_sha256':sha(guard),'commands':[],'source':[],'classes':[]}
src=OUT/'src/boundary';src.mkdir(parents=True);classes=OUT/'classes';classes.mkdir()
sources={
'ReturnLatchEdges':'''public static int edge(int n) { try { while(n>0) { if(n==1) return n; n--; } } catch(RuntimeException ignored) { return -1; } return 0; }''',
'NestedLatch':'''public static int nested(int n) { while(n>0) { if(n==1) return n; while(n>1) n--; n--; } return 0; }''',
'MultipleLatch':'''public static int multiple(int n) { while(n>0) { if(n==2) continue; if(n==1) return n; n--; } return 0; }'''}
for name,body in sources.items():
 f=src/(name+'.java');f.write_text('package boundary; public final class '+name+' { '+body+' }\n');record['source'].append({'path':str(f),'sha256':sha(f)})
assert run(OUT,'compile',[JDK/'bin/javac','-g','-source','8','-target','8','-proc:none','-d',classes,*sorted(src.glob('*.java'))])['exit_code']==0
for f in sorted((classes/'boundary').glob('*.class')):
 d=OUT/f.stem;d.mkdir();record['classes'].append({'path':str(f),'sha256':sha(f),'bytes':f.stat().st_size})
 assert run(d,'javap',[JDK/'bin/javap','-p','-c','-s','-v',f])['exit_code']==0
 assert run(d,'jarde-baseline',[CLI,'class-source','--input',f,'--class','boundary.'+f.stem,'--policy','single-class','--release','8','--format','json','--evidence','all'])['exit_code']==0
record['status']='observed-boundary-candidates-not-helper-acceptance';(OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n');print({'commands':len(record['commands']),'classes':len(record['classes']),'status':record['status']})
