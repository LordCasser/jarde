#!/usr/bin/env python3
import hashlib, json, shutil, subprocess, tempfile
from pathlib import Path

OUT = Path(__file__).resolve().parent / 'v3'
JDK = Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home')
JAVAC, JAVA, JAR, JAVAP = (JDK / 'bin/javac', JDK / 'bin/java', JDK / 'bin/jar', JDK / 'bin/javap')
JARDE = Path('/tmp/jarde-raw-receiver-final-v3-cli')
JADX = Path('/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx')
DRIVER = r'''import java.lang.reflect.*;
import java.util.*;
public class HeaderProbe {
  public static void main(String[] args) throws Exception {
    Class<?> c = Class.forName(args[0]);
    Method[] ms = c.getDeclaredMethods();
    Arrays.sort(ms, new Comparator<Method>() { public int compare(Method a, Method b) { return a.toString().compareTo(b.toString()); }});
    for (Method m : ms) {
      if (m.getName().equals("main")) continue;
      System.out.print("header=" + m.getName() + "<");
      for (TypeVariable<Method> v : m.getTypeParameters()) System.out.print(v.getName() + ":" + Arrays.toString(v.getBounds()) + "@" + v.getGenericDeclaration() + ";");
      System.out.print(">( ");
      for (Type t : m.getGenericParameterTypes()) System.out.print(t.getTypeName() + ";");
      System.out.println(") -> " + m.getGenericReturnType().getTypeName());
    }
    String probe = args[1];
    Object receiver = c.getConstructor().newInstance();
    Object marker = new Object();
    if (probe.equals("VoidDirect")) {
      c.getMethod("relay", Object.class).invoke(receiver, marker);
      System.out.println("behavior.marker=" + (c.getField("seen").get(receiver) == marker));
    } else if (probe.equals("ArrayRelay")) {
      Object[] value = new Object[]{marker};
      System.out.println("behavior.marker=" + (c.getMethod("relay", Object[].class).invoke(receiver, (Object)value) == value));
    } else if (probe.equals("NumberBoundRelay") || probe.equals("MethodShadow")) {
      Integer value = Integer.valueOf(17);
      System.out.println("behavior.marker=" + (c.getMethod("relay", Number.class).invoke(receiver, value) == value));
    } else if (probe.equals("IndependentCallee")) {
      System.out.println("behavior.marker=" + (c.getMethod("relay", Object.class).invoke(receiver, marker) == marker));
    } else if (probe.equals("MultiParam")) {
      System.out.println("behavior.marker=" + (c.getMethod("relay", Object.class, Object.class).invoke(receiver, marker, marker) == marker));
    } else if (probe.equals("RawReceiver") || probe.equals("MutatedReceiver")) {
      System.out.println("behavior.marker=" + (c.getMethod("relay", Object.class).invoke(receiver, marker) == marker));
    } else if (probe.equals("SameNameOverload")) {
      c.getMethod("relay", Object.class).invoke(receiver, marker);
      System.out.println("behavior.selected=" + c.getField("selected").get(receiver));
    } else if (probe.equals("SameErasureBinder")) {
      System.out.println("behavior.null=" + (c.getMethod("relay", Number.class).invoke(receiver, new Object[]{null}) == null));
    }
  }
}'''
CASES = {
'VoidDirect': '''public class VoidDirect<T> {
  public Object seen;
  public void sink(T x) { seen = x; }
  public void relay(T x) { sink(x); }
}''',
'ArrayRelay': '''public class ArrayRelay<T> {
  public T[] id(T[] x) { return x; }
  public T[] relay(T[] x) { return id(x); }
}''',
'NumberBoundRelay': '''public class NumberBoundRelay<T extends Number> {
  public T id(T x) { return x; }
  public T relay(T x) { return id(x); }
}''',
'MethodShadow': '''public class MethodShadow<T> {
  public <T extends Number> T relay(T x) { return this.<T>id(x); }
  public <U extends Number> U id(U x) { return x; }
}''',
'IndependentCallee': '''public class IndependentCallee<T> {
  public <U> U id(U x) { return x; }
  public T relay(T x) { return this.<T>id(x); }
}''',
'MultiParam': '''public class MultiParam<T> {
  public T first(T x, T y) { return x; }
  public T relay(T x, T y) { return first(x, y); }
}''',
'RawReceiver': '''public class RawReceiver<T> {
  @SuppressWarnings("rawtypes") public java.util.List box = new java.util.ArrayList();
  @SuppressWarnings("unchecked") public T relay(T x) { box.add(x); return (T) box.get(0); }
}''',
'SameNameOverload': '''public class SameNameOverload<T> {
  public String selected;
  public void pick(T x) { selected="generic"; }
  public void pick(String x) { selected="string"; }
  public void relay(T x) { pick(x); }
}''',
'SameErasureBinder': '''public class SameErasureBinder<T extends Number & Runnable> {
  public <U extends Number & Runnable> U sink(U x) { return x; }
  public T relay(T x) { return sink(x); }
}''',
'MutatedReceiver': '''public class MutatedReceiver<T> {
  @SuppressWarnings({"rawtypes","unchecked"}) public T relay(T x) {
    java.util.List<T> receiver = new java.util.ArrayList<T>();
    java.util.List raw = new java.util.ArrayList();
    raw.add(x);
    receiver = raw;
    return receiver.get(0);
  }
}'''
}
NEGATIVE = '''public class IncompatibleBinder<T extends Number> {
  public <U extends Number & Runnable> U sink(U x) { return x; }
  public T relay(T x) { return sink(x); }
}'''

def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def run(label, argv, cwd, save_dir, commands):
  argv=list(map(str,argv)); r=subprocess.run(argv,cwd=cwd,capture_output=True)
  (save_dir/(label+'.stdout')).write_bytes(r.stdout); (save_dir/(label+'.stderr')).write_bytes(r.stderr)
  commands.append({'label':label,'argv':argv,'cwd':str(cwd),'returncode':r.returncode,
    'stdout_sha256':sha(save_dir/(label+'.stdout')),'stderr_sha256':sha(save_dir/(label+'.stderr'))})
  return r

def main():
  if OUT.exists() and any(OUT.iterdir()): raise SystemExit('refusing to overwrite existing patrol evidence')
  OUT.mkdir(parents=True,exist_ok=True)
  commands=[]; cases=[]
  for name, source in CASES.items():
    area=OUT/name; area.mkdir()
    src=area/(name+'.java'); src.write_text(source+'\n')
    source_hash=sha(src)
    inclasses=area/'input-classes'; inclasses.mkdir()
    emptycp=area/'empty-input-classpath'; emptycp.mkdir()
    emptysp=area/'empty-input-sourcepath'; emptysp.mkdir()
    original_compile=run('original-javac',[JAVAC,'-source','8','-target','8','-g','-classpath',emptycp,'-sourcepath',emptysp,'-d',inclasses,src],area,area,commands)
    if original_compile.returncode != 0: raise SystemExit('fixture source failed to compile: '+name)
    jarpath=area/(name+'.jar')
    packed=run('freeze-jar',[JAR,'cf',jarpath,'-C',inclasses,'.'],area,area,commands)
    if packed.returncode: raise SystemExit('jar failed: '+name)
    javap=run('original-javap',[JAVAP,'-p','-c','-s','-v','-classpath',jarpath,name],area,area,commands)
    original_hashes={str(p.relative_to(inclasses)):sha(p) for p in sorted(inclasses.rglob('*.class'))}
    jarde_area=area/'jarde'; jarde_area.mkdir()
    jarde=run('jarde-class-source',[JARDE,'class-source','--input',jarpath,'--class',name,'--policy','plain-jar','--release','8','--format','text'],area,jarde_area,commands)
    jarde_src=jarde_area/(name+'.java'); jarde_src.write_bytes(jarde.stdout)
    jadx_area=area/'jadx'; jadx_area.mkdir(); jadx_out=jadx_area/'full-output'; jadx_out.mkdir()
    jadx=run('jadx',[JADX,'--no-res','-d',jadx_out,jarpath],area,jadx_area,commands)
    if jadx.returncode: raise SystemExit('JADX failed: '+name)
    candidates=list(jadx_out.rglob(name+'.java'))
    if len(candidates)!=1: raise SystemExit(f'{name}: expected one JADX source, got {len(candidates)}')
    jadx_src=jadx_area/(name+'.java'); shutil.copyfile(candidates[0],jadx_src)
    (area/'original.java').write_text(source+'\n')
    (area/'original.jar').write_bytes(jarpath.read_bytes())
    row={'name':name,'source':str(src),'source_sha256':source_hash,'input_jar':str(jarpath),'input_jar_sha256':sha(jarpath),
      'original_class_sha256':original_hashes,'jarde_source_sha256':sha(jarde_src),'jarde_cli_exit':jarde.returncode,
      'jadx_source_sha256':sha(jadx_src),'jadx_cli_exit':jadx.returncode,'flavors':{}}
    for flavor,sourcefile,qualified in [('original',src,name),('jarde',jarde_src,name),('jadx',jadx_src,'defpackage.'+name)]:
      row['flavors'][flavor]={'source_sha256':sha(sourcefile)}
      with tempfile.TemporaryDirectory(prefix='generic-call-'+flavor+'-') as tmp:
        tmp=Path(tmp); classes=tmp/'classes'; classes.mkdir(); cp=tmp/'empty-classpath'; cp.mkdir(); sp=tmp/'empty-sourcepath'; sp.mkdir()
        driver=tmp/'HeaderProbe.java'; driver.write_text(DRIVER+'\n')
        comp=run(flavor+'-javac',[JAVAC,'-source','8','-target','8','-g','-classpath',cp,'-sourcepath',sp,'-d',classes,sourcefile,driver],area,area,commands)
        fr={'compile_exit':comp.returncode,'compile_stderr_sha256':sha(area/(flavor+'-javac.stderr')),'compiled_class_sha256':{str(p.relative_to(classes)):sha(p) for p in sorted(classes.rglob('*.class'))}}
        if comp.returncode==0:
          probe=run(flavor+'-probe',[JAVA,'-Xverify:all','-cp',classes,'HeaderProbe',qualified,name],area,area,commands)
          probe_text=probe.stdout.decode(errors='replace')
          fr.update({'probe_exit':probe.returncode,
             'headers_stdout':'\n'.join(line for line in probe_text.splitlines() if line.startswith('header=')),
             'behavior_stdout':'\n'.join(line for line in probe_text.splitlines() if line.startswith('behavior.')),
             'probe_stdout_sha256':sha(area/(flavor+'-probe.stdout')),'probe_stderr_sha256':sha(area/(flavor+'-probe.stderr'))})
        row['flavors'][flavor]=fr
    cases.append(row)
  neg=OUT/'IncompatibleBinder.java'; neg.write_text(NEGATIVE+'\n')
  neg_area=OUT/'incompatible-binder-negative'; neg_area.mkdir()
  with tempfile.TemporaryDirectory(prefix='incompatible-binder-negative-') as tmp:
    tmp=Path(tmp); cp=tmp/'empty-classpath';cp.mkdir();sp=tmp/'empty-sourcepath';sp.mkdir();classes=tmp/'classes';classes.mkdir()
    nr=run('negative-original-javac',[JAVAC,'-source','8','-target','8','-g','-classpath',cp,'-sourcepath',sp,'-d',classes,neg],neg_area,neg_area,commands)
  manifest={'scope':'Corretto 8.432 debug exploratory only; ten whole-class cases plus one invalid-source preflight; no Cargo or production changes.',
    'jdk_home':str(JDK),'javac_sha256':sha(JAVAC),'java_sha256':sha(JAVA),'jar_sha256':sha(JAR),
    'javap_sha256':sha(JAVAP),
    'jarde_cli':str(JARDE),'jarde_cli_sha256':sha(JARDE),'jadx_cli':str(JADX),'jadx_cli_sha256':sha(JADX),
    'negative_control':{'source':str(neg),'source_sha256':sha(neg),'javac_exit':nr.returncode,'note':'intentionally invalid: class T extends Number cannot satisfy method U extends Number & Runnable; no class/JADX/Jarde pipeline exists'},
    'cases':cases,'commands':commands}
  manifest['files']=[{'path':str(p.relative_to(OUT)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(OUT.rglob('*')) if p.is_file() and p.name!='manifest.json']
  (OUT/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
  print(json.dumps({'cases':len(cases),'negative_javac_exit':nr.returncode,'results':str(OUT),'flavors':{c['name']:{f:{'compile':r.get('compile_exit'),'headers':r.get('headers_stdout','').strip(),'behavior':r.get('behavior_stdout','').strip()} for f,r in c['flavors'].items()} for c in cases}},ensure_ascii=False,indent=2))
if __name__=='__main__': main()
