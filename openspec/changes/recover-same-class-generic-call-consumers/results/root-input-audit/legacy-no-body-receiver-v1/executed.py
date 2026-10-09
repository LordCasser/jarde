import hashlib,json,pathlib,shutil,subprocess,tempfile,time
root=pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
out=root/'openspec/changes/recover-same-class-generic-call-consumers/results/root-input-audit/legacy-no-body-receiver-v1'
out.mkdir()
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
shutil.copy2(__file__,out/'executed.py')
name='GenericAbstractReceiverProbe'
source='public abstract class GenericAbstractReceiverProbe { public abstract <T extends Number> T target(T value); public static Number call(GenericAbstractReceiverProbe value) { return value.target(Integer.valueOf(7)); } public Number own() { return this.target(Integer.valueOf(8)); } }\n'
probe='public class Probe { static class Impl extends GenericAbstractReceiverProbe { public <T extends Number> T target(T value) { return value; } } public static void main(String[] args) throws Exception { java.lang.reflect.Method m=GenericAbstractReceiverProbe.class.getDeclaredMethod("target",Number.class); java.lang.reflect.TypeVariable<?> t=m.getTypeParameters()[0]; System.out.print(GenericAbstractReceiverProbe.call(new Impl())+":"+new Impl().own()+":"+t.getGenericDeclaration().equals(m)+":"+m.getGenericReturnType().equals(t)+":"+m.getGenericParameterTypes()[0].equals(t)+":"+java.util.Arrays.toString(t.getBounds())); } }\n'
(out/(name+'.java')).write_text(source);(out/'Probe.java').write_text(probe)
commands=[]
def run(label,argv):
    p=subprocess.run([str(x) for x in argv],cwd=root,capture_output=True)
    (out/(label+'.stdout')).write_bytes(p.stdout);(out/(label+'.stderr')).write_bytes(p.stderr)
    commands.append({'label':label,'argv':[str(x) for x in argv],'exit':p.returncode,'stdout_sha256':hashlib.sha256(p.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(p.stderr).hexdigest()})
    return p
javac=pathlib.Path(shutil.which('javac')).resolve();java=pathlib.Path(shutil.which('java')).resolve()
run('javac-version',[javac,'-version']);run('java-version',[java,'-version'])
input_dir=out/'input';input_dir.mkdir()
p=run('compile-input',[javac,'--release','8','-g:none','-classpath','','-sourcepath','','-d',input_dir,out/(name+'.java')]);assert p.returncode==0
cli_specs=[('baseline',pathlib.Path('/private/tmp/jarde-raw-receiver-final-v3-cli'),'3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70'),('candidate-v6',pathlib.Path('/private/tmp/jarde-generic-calls-candidate-v6-cli'),'c42c2f64df21fd74e22710e6887eac0040850d2e03906a202e051b65af1dc6d8')]
metrics=[]
for label,cli,sha in cli_specs:
    assert h(cli)==sha
    result=run(label+'-decompile',[cli,'class-source','--input',input_dir/(name+'.class'),'--class',name,'--policy','single-class','--release','8','--format','text'])
    area=out/label;area.mkdir();(area/(name+'.java')).write_bytes(result.stdout);(area/'Probe.java').write_text(probe)
    with tempfile.TemporaryDirectory(prefix='jarde-direct-return-context-') as tmp:
        classes=pathlib.Path(tmp)/'classes';classes.mkdir()
        compiled=run(label+'-compile',[javac,'--release','8','-classpath','','-sourcepath','','-d',classes,area/(name+'.java'),area/'Probe.java'])
        executed=run(label+'-verify',[java,'-Xverify:all','-cp',classes,'Probe']) if compiled.returncode==0 else None
        metrics.append({'label':label,'cli_sha256':sha,'compile_exit':compiled.returncode,'verify_exit':executed.returncode if executed else None,'probe':executed.stdout.decode() if executed else None,'emitted_source_sha256':h(area/(name+'.java')),'compiled_classes':{p.name:h(p) for p in classes.rglob('*.class')}})
files={str(p.relative_to(out)):h(p) for p in out.rglob('*') if p.is_file()}
data={'scope':'旧已接受abstract声明leaf在本类直接形参/explicit this的完整动态调用、method binder反射GC09保存性检查；单JDK no-debug，不冒充冻结四腿矩阵','source_snapshot':'source-snapshot-v38.json','tools':{str(javac):h(javac),str(java):h(java)},'commands':commands,'metrics':metrics,'files':files}
with (out/'manifest.json').open('x') as f:json.dump(data,f,indent=2,ensure_ascii=False);f.write('\n')
print(json.dumps(metrics,indent=2))
