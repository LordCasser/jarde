import hashlib,json,os,subprocess
from pathlib import Path
root=Path('/Users/lordcasser/workspace/projects/jarde');r=Path(__file__).resolve().parent
out=r/'nested-array-update-baseline-v1';assert not out.exists();out.mkdir()
cli=Path('/private/tmp/jarde-bigdecimal-number-cli-v1');metadata=json.loads((r/'candidate-cli-v1.json').read_text())
def ident(p):
 p=Path(p);return {'path':str(p),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
assert ident(cli)['sha256']==metadata['cli_sha256']
env={k:v for k,v in os.environ.items() if k not in ['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH']};commands=[];cases=[]
original_source=root/'tests/fixtures/recover-lambda-primitive-array-capture/P02_multianewarray.java'
for leg,v,jdk in [('javac8','v8','/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),('javac23','v23','/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home')]:
 d=out/leg;d.mkdir();orig=d/'original';orig.mkdir();sources=d/'source';sources.mkdir();classes=d/'classes';classes.mkdir();empty=d/'empty';empty.mkdir()
 original=root/'tests/fixtures/recover-lambda-primitive-array-capture'/v/'P02_multianewarray.class';(orig/original.name).write_bytes(original.read_bytes())
 def cmd(label,args):
  args=[str(x) for x in args];p=subprocess.run(args,env=env,cwd=root,capture_output=True);a=d/(label+'.stdout');b=d/(label+'.stderr');a.write_bytes(p.stdout);b.write_bytes(p.stderr);commands.append({'leg':leg,'label':label,'argv':args,'exit':p.returncode,'stdout':ident(a),'stderr':ident(b)});return p
 java=Path(jdk)/'bin/java';javac=Path(jdk)/'bin/javac';javap=Path(jdk)/'bin/javap'
 oracle=cmd('original-run',[java,'-Xverify:all','-cp',orig,'P02_multianewarray']);assert oracle.returncode==0 and oracle.stdout==b'6\n' and oracle.stderr==b''
 cmd('original-javap',[javap,'-p','-c','-s',original])
 report=cmd('candidate-render',[cli,'class-source','--input',original,'--class','P02_multianewarray','--policy','single-class','--release','8','--format','json','--evidence','all']);assert report.returncode==0
 doc=json.loads(report.stdout);source=sources/'P02_multianewarray.java';source.write_text(doc['text'])
 compile=cmd('candidate-compile',[javac,'-source','8','-target','8','-g:none','-Xlint:-options','-classpath',empty,'-sourcepath',empty,'-d',classes,source])
 run=cmd('candidate-run',[java,'-Xverify:all','-cp',classes,'P02_multianewarray']) if compile.returncode==0 else None
 members=[]
 for m in doc['methods']:
  b=m['outcome'].get('report',{});members.append({'name':bytes(m['item']['name']['raw']).decode(),'descriptor':bytes(m['item']['descriptor']['raw']).decode(),'outcome':m['outcome']['kind'],'quality':b.get('quality'),'representation':b.get('representation'),'body':b.get('text'),'diagnostics':b.get('diagnostics')})
 cases.append({'leg':leg,'input':ident(original),'original_class_copy':ident(orig/original.name),'tools':{k:ident(p) for k,p in [('java',java),('javac',javac),('javap',javap)]},'original_exit':oracle.returncode,'candidate_compile_exit':compile.returncode,'candidate_run_exit':None if run is None else run.returncode,'raw_streams_and_exit_match':run is not None and run.returncode==oracle.returncode and run.stdout==oracle.stdout and run.stderr==oracle.stderr,'members':members,'generated_class_set':[p.relative_to(classes).as_posix() for p in sorted(classes.rglob('*.class'))]})
files=[ident(p)|{'path':p.relative_to(out).as_posix()} for p in sorted(out.rglob('*')) if p.is_file()]
result={'schema':'fresh-nested-array-update-baseline-v1','cli':ident(cli),'candidate_sources':metadata['candidate_sources'],'source':ident(original_source),'original_execution_fresh':True,'candidate_execution_fresh':True,'jadx_execution_or_extraction_fresh':False,'cases':cases,'commands':commands,'files':files,'scope':'DT26 capture already recovered; inspect exact nested row compound-update helper, not a new capture mechanism'}
(out/'manifest.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps([{'leg':c['leg'],'compile':c['candidate_compile_exit'],'run':c['candidate_run_exit'],'matches':c['raw_streams_and_exit_match']} for c in cases]))
