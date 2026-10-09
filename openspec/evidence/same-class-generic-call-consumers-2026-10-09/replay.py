#!/usr/bin/env python3
"""Freeze or replay the GC-01..GC-08 same-class generic-call corpus.

First invocation creates immutable inputs under --frozen-inputs. Later runs reuse
those jars and require a fresh --out. No generated classes survive a run.
"""
import argparse
import hashlib
import json
import shutil
import subprocess
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
SOURCE_ROOT = EVIDENCE / 'fixtures' / 'sources'
INDEX_SOURCE = EVIDENCE / 'frozen-input-index.json'
JDKS = {
    'corretto8': {'home': Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
                  'release': ['-source', '8', '-target', '8']},
    'openjdk23': {'home': Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'),
                  'release': ['--release', '8']},
}
DEBUGS = {'debug': ['-g'], 'nodebug': ['-g:none']}
FROZEN_OLD = REPO / 'openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen'
FROZEN_BOUND = REPO / 'openspec/evidence/published-generic-call-adaptation-patrol-2026-10-09/overload-control/strict-results'

PROBE = r'''import java.lang.reflect.*;
import java.util.*;
import java.io.*;
public class FixtureProbe {
  static final class NumberRunnable extends Number implements Runnable {
    private static final long serialVersionUID=1L;
    public int intValue(){return 17;} public long longValue(){return 17L;}
    public float floatValue(){return 17.0f;} public double doubleValue(){return 17.0d;}
    public void run(){}
  }
  static int checks=0, failures=0;
  static void check(boolean ok, String label) {
    checks++; if (!ok) { failures++; System.out.println("probe.fail="+label); }
  }
  static String simple(Class<?> c) {
    if (c.isArray()) return simple(c.getComponentType())+"[]";
    if (c.getName().startsWith("java.")) return c.getName();
    return c.getSimpleName();
  }
  static String methodKey(Method m) {
    StringBuilder b=new StringBuilder(m.getName()).append('(');
    for (Class<?> p:m.getParameterTypes()) b.append(simple(p)).append(';');
    return b.append(')').append("->").append(simple(m.getReturnType())).toString();
  }
  static int variableIndex(TypeVariable<?> v, TypeVariable<?>[] all) {
    for(int i=0;i<all.length;i++) if(all[i].equals(v)) return i;
    return -1;
  }
  static String canonical(Type t, Class<?> owner) {
    if (t instanceof Class<?>) return simple((Class<?>)t);
    if (t instanceof GenericArrayType) return canonical(((GenericArrayType)t).getGenericComponentType(),owner)+"[]";
    if (t instanceof ParameterizedType) {
      ParameterizedType p=(ParameterizedType)t; StringBuilder b=new StringBuilder(canonical(p.getRawType(),owner)).append('<');
      Type[] a=p.getActualTypeArguments(); for(int i=0;i<a.length;i++){ if(i>0)b.append(','); b.append(canonical(a[i],owner)); }
      return b.append('>').toString();
    }
    if (t instanceof TypeVariable<?>) {
      TypeVariable<?> v=(TypeVariable<?>)t; GenericDeclaration d=v.getGenericDeclaration(); int index=-1; String key="UNKNOWN";
      if (d instanceof Class<?> && d==owner) { index=variableIndex(v,owner.getTypeParameters()); key="CLASS#"+index; check(index>=0,"class-var-owner-index:"+v.getName()); }
      else if (d instanceof Method) {
        Method found=null; for(Method m:owner.getDeclaredMethods()) if(m.equals(d)){found=m;break;}
        if(found!=null){index=variableIndex(v,found.getTypeParameters());key="METHOD#"+methodKey(found)+"#"+index;check(index>=0 && found.equals(d),"method-var-exact-declaration:"+v.getName());}
        else check(false,"method-var-declaration-not-in-target:"+v.getName());
      } else if (d instanceof Constructor<?>) {
        Constructor<?> found=null; for(Constructor<?> k:owner.getDeclaredConstructors()) if(k.equals(d)){found=k;break;}
        if(found!=null){index=variableIndex(v,found.getTypeParameters());key="CTOR#"+Arrays.toString(found.getParameterTypes())+"#"+index;check(index>=0 && found.equals(d),"ctor-var-exact-declaration:"+v.getName());}
        else check(false,"ctor-var-declaration-not-in-target:"+v.getName());
      } else check(false,"foreign-type-variable:"+v.getName());
      return key;
    }
    return t.getTypeName();
  }
  static String bounds(TypeVariable<?> v, Class<?> owner) {
    StringBuilder b=new StringBuilder(); Type[] bs=v.getBounds();
    for(int i=0;i<bs.length;i++){if(i>0)b.append('&');b.append(canonical(bs[i],owner));}
    return b.toString();
  }
  static void headers(Class<?> c) {
    TypeVariable<?>[] cv=c.getTypeParameters();
    for(int i=0;i<cv.length;i++){
      TypeVariable<?> v=cv[i]; check(v.getGenericDeclaration()==c,"class-formal-owner:"+i);
      check(variableIndex(v,cv)==i,"class-formal-index:"+i);
      System.out.println("classformal#"+i+":"+v.getName()+" bounds="+bounds(v,c));
    }
    Field[] fs=c.getDeclaredFields(); Arrays.sort(fs,(a,b)->a.getName().compareTo(b.getName()));
    for(Field f:fs) System.out.println("field:"+f.getName()+"="+canonical(f.getGenericType(),c));
    Constructor<?>[] cs=c.getDeclaredConstructors(); Arrays.sort(cs,(a,b)->a.toString().compareTo(b.toString()));
    for(Constructor<?> k:cs){
      Type[] ps=k.getGenericParameterTypes(); StringBuilder b=new StringBuilder("constructor(");
      for(int i=0;i<ps.length;i++){if(i>0)b.append(';');b.append(canonical(ps[i],c));}
      System.out.println(b.append(")").toString());
      TypeVariable<?>[] vs=k.getTypeParameters(); for(int i=0;i<vs.length;i++){TypeVariable<?> v=vs[i];check(v.getGenericDeclaration().equals(k),"ctor-formal-exact-declaration:"+i);check(variableIndex(v,vs)==i,"ctor-formal-index:"+i);System.out.println("ctorformal#"+i+" bounds="+bounds(v,c));}
    }
    Method[] ms=c.getDeclaredMethods(); Arrays.sort(ms,(a,b)->methodKey(a).compareTo(methodKey(b)));
    for(Method m:ms){
      TypeVariable<Method>[] vs=m.getTypeParameters();
      for(int i=0;i<vs.length;i++){TypeVariable<?> v=vs[i];check(v.getGenericDeclaration().equals(m),"method-formal-exact-declaration:"+methodKey(m)+"#"+i);check(variableIndex(v,vs)==i,"method-formal-index:"+methodKey(m)+"#"+i);System.out.println("methodformal:"+methodKey(m)+"#"+i+" bounds="+bounds(v,c));}
      StringBuilder b=new StringBuilder("method:"+methodKey(m)+" return="+canonical(m.getGenericReturnType(),c)+" params=");
      Type[] ps=m.getGenericParameterTypes(); for(int i=0;i<ps.length;i++){if(i>0)b.append(';');b.append(canonical(ps[i],c));}
      System.out.println(b);
    }
  }
  static Method method(Class<?> c,String n,Class<?>... ps)throws Exception{return c.getDeclaredMethod(n,ps);}
  static Object call(Class<?> c,Object o,String n,Class<?>[] ps,Object... args)throws Exception{return method(c,n,ps).invoke(o,args);}
  static void behavior(String n,Class<?> c)throws Exception{
    Object m=new Object(), o;
    if(n.equals("EmptySink")){o=c.getConstructor().newInstance();call(c,o,"sink",new Class[]{Object.class},m);System.out.println("behavior=void-call");}
    else if(n.equals("VoidDirect")){o=c.getConstructor().newInstance();call(c,o,"relay",new Class[]{Object.class},m);check(c.getField("seen").get(o)==m,"void-marker");check(c.getField("calls").getInt(o)==1,"void-call-count");System.out.println("behavior=marker+call-count");}
    else if(n.equals("FieldSetter")){o=c.getConstructor().newInstance();call(c,o,"set",new Class[]{Object.class},m);check(c.getField("value").get(o)==m,"field-marker");System.out.println("behavior=field-marker");}
    else if(n.equals("CallRelay")){o=c.getConstructor().newInstance();check(call(c,o,"relay",new Class[]{Object.class},m)==m,"relay-marker");System.out.println("behavior=marker");}
    else if(n.equals("NullCall")){o=c.getConstructor().newInstance();check(call(c,o,"relay",new Class[]{})==null,"null-result");System.out.println("behavior=null");}
    else if(n.equals("ArrayRelay")){o=c.getConstructor().newInstance();Object[] a=new Object[]{m};check(call(c,o,"relay",new Class[]{Object[].class},(Object)a)==a,"array-identity");System.out.println("behavior=array-identity");}
    else if(n.equals("NumberBoundRelay")){o=c.getConstructor().newInstance();Integer v=Integer.valueOf(17);check(call(c,o,"relay",new Class[]{Number.class},v)==v,"number-marker-identity");check(((Number)v).intValue()==17,"number-17");System.out.println("behavior=Number17+identity");}
    else if(n.equals("MultiParam")){o=c.getConstructor().newInstance();Object a=new Object(),b=new Object();check(call(c,o,"relay",new Class[]{Object.class,Object.class},a,b)==a,"multiparam-position");System.out.println("behavior=first-argument");}
    else if(n.equals("WideRelay")){o=c.getConstructor().newInstance();Object a=new Object(),b=new Object();check(call(c,o,"relay",new Class[]{long.class,Object.class,double.class,Object.class},Long.valueOf(71),a,Double.valueOf(2.5),b)==b,"wide-slot-position");System.out.println("behavior=wide-slot-second-marker");}
    else if(n.equals("TwoClassVariables")){o=c.getConstructor().newInstance();Object b=new Object();check(call(c,o,"relay",new Class[]{Object.class,Object.class},"A",b)==b,"two-formal-position");System.out.println("behavior=class-formal-B-marker");}
    else if(n.equals("ArrayDimensionRelay")){o=c.getConstructor().newInstance();Object[][] a=new Object[][]{{m}};check(call(c,o,"relay",new Class[]{Object[][].class},(Object)a)==a,"array2d-identity");System.out.println("behavior=array2d-identity");}
    else if(n.equals("DeepRelay")||n.equals("ReverseDeclarationRelay")){o=c.getConstructor().newInstance();check(call(c,o,"relay0",new Class[]{Object.class},m)==m,"deep-relay-marker");System.out.println("behavior=deep-marker");}
    else if(n.equals("UnknownIncoming")){o=c.getConstructor().newInstance();check(call(c,o,"safe",new Class[]{Object.class},m)==m,"safe-incoming");check(call(c,o,"unsafe",new Class[]{Object.class},m)==m,"unchecked-cast-incoming");check(call(c,o,"untouched",new Class[]{Object.class},m)==m,"independent-leaf");System.out.println("behavior=safe+unchecked+independent");}
    else if(n.equals("IndependentLeaf")){o=c.getConstructor().newInstance();check(call(c,o,"leaf",new Class[]{Object.class},m)==m,"leaf-marker");System.out.println("behavior=leaf-marker");}
    else if(n.equals("MethodShadow")){o=c.getConstructor().newInstance();Integer v=17;check(call(c,o,"relay",new Class[]{Number.class},v)==v,"shadow-number17");System.out.println("behavior=Number17+identity");}
    else if(n.equals("IndependentCallee")){o=c.getConstructor().newInstance();check(call(c,o,"relay",new Class[]{Object.class},m)==m,"independent-callee-marker");System.out.println("behavior=marker");}
    else if(n.equals("CompatibleIntersectionBinder")){o=c.getConstructor().newInstance();NumberRunnable v=new NumberRunnable();Object got=call(c,o,"relay",new Class[]{Number.class},v);check(got==v,"compatible-intersection-marker");check(((Number)got).intValue()==17,"compatible-intersection-number17");check(call(c,o,"relay",new Class[]{Number.class},new Object[]{null})==null,"compatible-null-control");System.out.println("behavior=Number17+identity+null-control");}
    else if(n.equals("SameErasureBinder")){o=c.getConstructor().newInstance();call(c,o,"useNull",new Class[]{});check(c.getField("calls").getInt(o)==1,"same-erasure-method-invoked");System.out.println("behavior=null-call-count=1");}
    else if(n.equals("TypedReceiverRelay")){o=c.getConstructor().newInstance();check(call(c,o,"relay",new Class[]{c,Object.class},o,m)==m,"typed-receiver-marker");System.out.println("behavior=typed-receiver-marker");}
    else if(n.equals("RawOwnReceiver")||n.equals("ReboundOwnReceiver")){o=c.getConstructor().newInstance();check(call(c,o,"relay",new Class[]{Object.class},m)==m,"raw-receiver-marker");System.out.println("behavior=raw-receiver-marker");}
    else if(n.equals("CallHold")||n.equals("ExceptionHold")){o=c.getConstructor(Object.class).newInstance(m);check(c.getField("v").get(o)==m,"constructor-field-marker");System.out.println("behavior=constructor-marker");}
    else if(n.equals("CatchCallMarker")){Object normal=c.getConstructor(Object.class,boolean.class).newInstance(m,false);check(c.getField("value").get(normal)==m,"catch-normal-marker");Object thrown=c.getConstructor(Object.class,boolean.class).newInstance(m,true);check(c.getField("value").get(thrown)==null,"catch-throw-null");System.out.println("behavior=normal-marker+caught-throw");}
    else if(n.equals("BoundOverload")){o=c.getConstructor().newInstance();PrintStream old=System.out;ByteArrayOutputStream buf=new ByteArrayOutputStream();System.setOut(new PrintStream(buf));try{call(c,o,"relay",new Class[]{Number.class},Integer.valueOf(17));}finally{System.setOut(old);}String s=new String(buf.toByteArray(),"UTF-8").trim();check(s.equals("number"),"bound-overload-target-number:"+s);System.out.println("behavior=target-number:"+s);}
    else if(n.equals("SameNameOverload")){o=c.getConstructor().newInstance();call(c,o,"relay",new Class[]{Object.class},m);check(c.getField("selected").get(o).equals("generic"),"same-name-target-generic");System.out.println("behavior=target-generic");}
    else if(n.equals("PlainUpperBoundOverload")){o=c.getConstructor().newInstance();call(c,o,"relay",new Class[]{Number.class},Integer.valueOf(17));check(c.getField("selected").get(o).equals("number"),"plain-bound-target-number");System.out.println("behavior=target-number");}
    else if(n.equals("CycleRelay")){o=c.getConstructor().newInstance();check(call(c,o,"left",new Class[]{Object.class,boolean.class},m,true)==m,"finite-cycle-marker");System.out.println("behavior=finite-cycle-marker");}
    else if(n.equals("MethodHandleUse")){o=c.getConstructor().newInstance();check(call(c,o,"relay",new Class[]{Object.class},m)==m,"method-handle-marker");System.out.println("behavior=method-handle-marker");}
    else if(n.equals("MultiUseResult")){o=c.getConstructor().newInstance();check(call(c,o,"relay",new Class[]{Object.class},m)==m,"multiuse-return-marker");check(c.getField("observed").get(o)==m,"multiuse-observer-marker");System.out.println("behavior=return+observer-marker");}
    else if(n.equals("IncompleteSite")){o=c.getConstructor().newInstance();check(call(c,o,"relay",new Class[]{Object.class,boolean.class},m,true)==m,"conditional-call-marker");check(call(c,o,"relay",new Class[]{Object.class,boolean.class},m,false)==m,"conditional-direct-marker");System.out.println("behavior=both-conditional-paths");}
    else if(n.equals("VarargsCall")){o=c.getConstructor().newInstance();check(call(c,o,"relay",new Class[]{Object.class},m)==m,"varargs-marker");System.out.println("behavior=varargs-marker");}
    else if(n.equals("BridgeUnknown")){o=c.getConstructor().newInstance();String v="marker";check(call(c,o,"relay",new Class[]{Object.class},v).equals(v),"bridge-marker");boolean bridge=false;for(Method x:c.getDeclaredMethods())if(x.isBridge())bridge=true;check(bridge,"bridge-present");System.out.println("behavior=bridge-marker");}
    else if(n.equals("InheritedUnknown")){o=c.getConstructor().newInstance();c.getMethod("add",Object.class).invoke(o,m);check(call(c,o,"relay",new Class[]{int.class},0)==m,"inherited-marker");System.out.println("behavior=inherited-marker");}
    else throw new IllegalArgumentException("unknown probe "+n);
  }
  public static void main(String[] args)throws Exception{
    Class<?> c=Class.forName(args[0]); String n=args[1]; headers(c);
    try{behavior(n,c);}catch(Throwable t){failures++;System.out.println("probe.behavior-error="+t.getClass().getName()+":"+t.getMessage());}
    System.out.println("probe.checks="+checks);System.out.println("probe.failures="+failures);
    if(failures!=0)System.exit(1);
  }
}'''

def sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def run(label, argv, cwd, dest, commands):
    argv=list(map(str,argv)); p=subprocess.run(argv,cwd=cwd,capture_output=True)
    out=dest/(label+'.stdout'); err=dest/(label+'.stderr')
    out.write_bytes(p.stdout); err.write_bytes(p.stderr)
    commands.append({'label':label,'argv':argv,'cwd':str(cwd),'returncode':p.returncode,
                     'stdout_sha256':sha(out),'stderr_sha256':sha(err)})
    return p

def load_source_index():
    data=json.loads(INDEX_SOURCE.read_text())
    return data

def freeze_inputs(root, source_index, commands):
    if root.exists() and any(root.iterdir()): raise SystemExit(f'refusing to overwrite frozen inputs: {root}')
    root.mkdir(parents=True,exist_ok=True)
    source_root=root/'source-files'; source_root.mkdir()
    rows=[]
    by_name={x['name']:x for x in source_index['new_fixtures']}
    for leg,config in JDKS.items():
      home=config['home']; javac=home/'bin/javac'; jar=home/'bin/jar'; javap=home/'bin/javap'
      for debug,debug_args in DEBUGS.items():
        for name,item in by_name.items():
          area=root/leg/debug/name; area.mkdir(parents=True)
          source=REPO/item['source']; copied=area/source.name; shutil.copyfile(source,copied)
          srcs=[copied]
          cp=area/'empty-input-classpath';cp.mkdir();sp=area/'empty-input-sourcepath';sp.mkdir()
          with tempfile.TemporaryDirectory(prefix='gc-freeze-') as td:
            tmp=Path(td); classes=tmp/'classes';classes.mkdir()
            cc=run('freeze-javac',[javac,*config['release'],*debug_args,'-classpath',cp,'-sourcepath',sp,'-d',classes,*srcs],area,area,commands)
            if cc.returncode: raise SystemExit(f'new fixture input did not compile: {leg}/{debug}/{name}')
            jarpath=area/(name+'.jar')
            j=run('freeze-jar',[jar,'cf',jarpath,'-C',classes,'.'],area,area,commands)
            if j.returncode: raise SystemExit('jar failed: '+name)
            class_hashes={str(p.relative_to(classes)):sha(p) for p in sorted(classes.rglob('*.class'))}
          jp=run('input-javap',[javap,'-p','-c','-s','-v','-classpath',jarpath,name],area,area,commands)
          rows.append({'name':name,'group':item['group'],'leg':leg,'debug':debug,'kind':'new-source',
                       'source':[str(copied.relative_to(root))],'source_sha256':[sha(copied)],'jar':str(jarpath.relative_to(root)),
                       'jar_sha256':sha(jarpath),'class_sha256':class_hashes,'javap_exit':jp.returncode,
                       'source_original_path':str(source.relative_to(REPO))})
    for item in source_index['reused_frozen_inputs']:
      leg,debug,name=item['leg'],item['debug'],item['name']; area=root/leg/debug/name;area.mkdir(parents=True,exist_ok=True)
      old_src=REPO/item['source']; old_jar=REPO/item['jar']
      copied_src=area/old_src.name; copied_jar=area/(name+'.jar')
      shutil.copyfile(old_src,copied_src);shutil.copyfile(old_jar,copied_jar)
      if sha(copied_jar)!=item['jar_sha256']: raise SystemExit('reused historical input hash changed: '+name)
      rows.append({'name':name,'group':item['group'],'leg':leg,'debug':debug,'kind':'reused-existing-jar',
                   'source':[str(copied_src.relative_to(root))],'source_sha256':[sha(copied_src)],'jar':str(copied_jar.relative_to(root)),
                   'jar_sha256':sha(copied_jar),'source_original_path':item['source'],'jar_original_path':item['jar'],
                   'reuse_note':item['reuse']})
      javap=JDKS[leg]['home']/'bin/javap'
      jp=run('input-javap',[javap,'-p','-c','-s','-v','-classpath',copied_jar,name],area,area,commands)
      rows[-1]['javap_exit']=jp.returncode
    manifest={'schema':'gc-frozen-inputs-v1','source_index':str(INDEX_SOURCE.relative_to(REPO)),
              'source_index_sha256':sha(INDEX_SOURCE),'inputs':rows,'commands':commands}
    manifest['files']=[{'path':str(p.relative_to(root)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(root.rglob('*')) if p.is_file() and p.name!='frozen-input-manifest.json']
    (root/'frozen-input-manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    return manifest

def resolve_classname(source_files,name):
    for p in source_files:
      text=p.read_text(errors='replace')
      package=None
      for line in text.splitlines():
        line=line.strip()
        if line.startswith('package ') and line.endswith(';'): package=line[8:-1]
      if name in p.stem:
        return (package+'.' if package else '')+name
    return name

def build_and_replay(out, frozen, cli, index, cli_label, only_reused=False, selected_names=None, selected_leg=None, selected_debug=None):
    if out.exists(): raise SystemExit(f'refusing to overwrite results: {out}')
    out.mkdir(parents=True)
    runner_snapshot=out/'replay-executed.py'
    shutil.copyfile(Path(__file__).resolve(),runner_snapshot)
    runner_snapshot_sha256=sha(runner_snapshot)
    jarde=Path(cli).resolve(); jadx=Path(index['reference_jadx']).resolve()
    if not jadx.is_file(): raise SystemExit(f'missing JADX launcher: {jadx}')
    libdir=jadx.parent.parent/'lib'; libs=sorted(libdir.glob('*.jar'))
    if len(libs)!=57: raise SystemExit(f'expected 57 JADX libs, found {len(libs)}')
    commands=[]; results=[]
    frozen_manifest=json.loads((frozen/'frozen-input-manifest.json').read_text())
    input_keys=[(r['name'],r['leg'],r['debug']) for r in frozen_manifest['inputs']]
    if len(input_keys)!=len(set(input_keys)):
      raise SystemExit(f'frozen manifest has duplicate input keys: rows={len(input_keys)} unique={len(set(input_keys))}')
    expected={(name,leg,debug) for name in {k[0] for k in input_keys} for leg in JDKS for debug in DEBUGS}
    if set(input_keys)!=expected:
      raise SystemExit(f'frozen input coverage mismatch: actual={len(set(input_keys))} expected={len(expected)}')
    shared_probe=out/'FixtureProbe.java';shared_probe.write_text(PROBE+'\n')
    probe_sha=sha(shared_probe)
    selected_inputs=[x for x in frozen_manifest['inputs'] if (not only_reused or x.get('kind')=='reused-existing-jar') and (not selected_names or x['name'] in selected_names) and (not selected_leg or x['leg']==selected_leg) and (not selected_debug or x['debug']==selected_debug)]
    for leg,config in JDKS.items():
      if selected_leg and leg!=selected_leg: continue
      home=config['home']; javac=home/'bin/javac'; java=home/'bin/java'; javap=home/'bin/javap'
      for debug,debug_args in DEBUGS.items():
        if selected_debug and debug!=selected_debug: continue
        for fixture in [x for x in selected_inputs if x['leg']==leg and x['debug']==debug]:
          name=fixture['name']; input_area=frozen/leg/debug/name; area=out/leg/debug/name;area.mkdir(parents=True)
          copytree=area/'input-source';copytree.mkdir()
          sourcepaths=[input_area/Path(x).name for x in []]
          input_row=None
          for r in frozen_manifest['inputs']:
            if r['leg']==leg and r['debug']==debug and r['name']==name: input_row=r;break
          if input_row is None: raise SystemExit(f'no frozen input row {leg}/{debug}/{name}')
          sources=[]
          for s in input_row['source']:
            src=frozen/s; dst=copytree/Path(s).name;shutil.copyfile(src,dst);sources.append(dst)
          jar=frozen/input_row['jar'];shutil.copyfile(jar,area/(name+'.input.jar'))
          (area/'FixtureProbe.java').write_text(PROBE+'\n')
          # Jarde class-source for every top-level input class, preserving helper classes.
          jar_names=[]
          with tempfile.TemporaryDirectory(prefix='gc-jar-list-') as td:
            listed=subprocess.run([home/'bin/jar','tf',jar],capture_output=True)
            (area/'jar-list.stdout').write_bytes(listed.stdout);(area/'jar-list.stderr').write_bytes(listed.stderr)
            commands.append({'label':'jar-list','argv':[str(home/'bin/jar'),'tf',str(jar)],'cwd':str(area),'returncode':listed.returncode,'stdout_sha256':sha(area/'jar-list.stdout'),'stderr_sha256':sha(area/'jar-list.stderr')})
            jar_names=[x[:-6].replace('/','.') for x in listed.stdout.decode(errors='replace').splitlines() if x.endswith('.class') and '$' not in x]
          # full JADX source tree
          jadx_area=area/'jadx';jadx_area.mkdir();jadx_out=jadx_area/'full-output';jadx_out.mkdir()
          jr=run('jadx',[jadx,'--no-res','-d',jadx_out,jar],area,jadx_area,commands)
          jadx_sources=sorted(jadx_out.rglob('*.java'))
          copied_jadx=[]
          for p in jadx_sources:
            rel=p.relative_to(jadx_out);dst=jadx_area/'compile-tree'/rel;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dst);copied_jadx.append(dst)
          # Jarde emits one whole-class file per physical top-level class in the jar.
          jarde_area=area/'jarde';jarde_area.mkdir();jarde_sources=[]
          jarde_status=[]
          for target in jar_names:
            rr=run('jarde-'+target.replace('.','_'),[jarde,'class-source','--input',jar,'--class',target,'--policy','plain-jar','--release','8','--format','text'],area,jarde_area,commands)
            dest=jarde_area/(target.replace('.','_')+'.java');dest.write_bytes(rr.stdout);jarde_sources.append(dest)
            jarde_status.append({'class':target,'exit':rr.returncode,'source':str(dest.relative_to(out)),'source_sha256':sha(dest)})
          flavors={'original':sources,'jadx':copied_jadx,cli_label:jarde_sources}
          declaration_heads=[]
          for source in jarde_sources:
            for line in source.read_text(errors='replace').splitlines():
              line=line.strip()
              if line.startswith(('public class ','class ','public interface ','interface ','public enum ','enum ')):
                declaration_heads.append(line)
          head_assertion=bool(declaration_heads)
          row={'name':name,'group':fixture['group'],'leg':leg,'debug':debug,'input_jar_sha256':sha(area/(name+'.input.jar')),
               'input_source_sha256':[sha(x) for x in sources],'probe_sha256':sha(area/'FixtureProbe.java'),'jarde_cli_sha256':sha(jarde),
               'jadx_source_sha256':[sha(x) for x in copied_jadx],'jarde_sources':jarde_status,
               'jarde_declaration_heads':declaration_heads,'jarde_nonempty_declaration_head_assertion':head_assertion,'flavors':{}}
          classnames={'original':name,'jadx':resolve_classname(copied_jadx,name),cli_label:name}
          for flavor,srcfiles in flavors.items():
            flavor_dir=area/'compile-sources'/flavor;flavor_dir.mkdir(parents=True)
            for src in srcfiles:
              if flavor=='jadx':
                dst=flavor_dir/src.relative_to(jadx_area/'compile-tree')
              elif flavor=='original' and fixture.get('kind')=='reused-existing-jar' and len(srcfiles)==1:
                dst=flavor_dir/(name+'.java')
              else:
                dst=flavor_dir/src.name
              dst.parent.mkdir(parents=True,exist_ok=True)
              if dst.exists(): raise SystemExit(f'duplicate source basename in {flavor}/{name}: {src.name}')
              shutil.copyfile(src,dst)
            flavor_sources=sorted(flavor_dir.rglob('*.java'))
            result={'source_sha256':[sha(x) for x in flavor_sources]}
            with tempfile.TemporaryDirectory(prefix='gc-compile-') as td:
              tmp=Path(td);classes=tmp/'classes';classes.mkdir();cp=tmp/'empty-classpath';cp.mkdir();sp=tmp/'empty-sourcepath';sp.mkdir()
              compiled=run(flavor+'-javac',[javac,*config['release'],*debug_args,'-classpath',cp,'-sourcepath',sp,'-d',classes,*flavor_sources,area/'FixtureProbe.java'],area,flavor_dir,commands)
              result['compile_exit']=compiled.returncode
              result['compile_stdout_sha256']=sha(flavor_dir/(flavor+'-javac.stdout'));result['compile_stderr_sha256']=sha(flavor_dir/(flavor+'-javac.stderr'))
              result['compiled_classes_sha256']={str(p.relative_to(classes)):sha(p) for p in sorted(classes.rglob('*.class'))}
              if compiled.returncode==0:
                runtime=run(flavor+'-probe',[java,'-Xverify:all','-cp',classes,'FixtureProbe',classnames[flavor],name],area,flavor_dir,commands)
                result['probe_exit']=runtime.returncode;result['probe_stdout_sha256']=sha(flavor_dir/(flavor+'-probe.stdout'));result['probe_stderr_sha256']=sha(flavor_dir/(flavor+'-probe.stderr'))
                result['probe_stdout']=runtime.stdout.decode(errors='replace')
                headers=[line for line in result['probe_stdout'].splitlines() if line.startswith(('classformal#','field:','constructor(','ctorformal#','methodformal:','method:'))]
                result['reflection_sha256']=hashlib.sha256(('\n'.join(headers)+'\n').encode()).hexdigest()
                result['behavior_lines']=[line for line in result['probe_stdout'].splitlines() if line.startswith('behavior=')]
                result['probe_failures']=[line for line in result['probe_stdout'].splitlines() if line.startswith(('probe.fail=','probe.behavior-error='))]
            row['flavors'][flavor]=result
          original=row['flavors']['original'].get('reflection_sha256')
          for flavor in ('jadx',cli_label):
            row['flavors'][flavor]['reflection_matches_original']=(row['flavors'][flavor].get('reflection_sha256')==original) if original and row['flavors'][flavor].get('reflection_sha256') else None
          results.append(row)
    jversion=run('jadx-version',[jadx,'--version'],out,out,commands)
    manifest={'scope':'Frozen four-leg whole-class baseline replay; exploratory outputs remain separate.',
      'accepted_baseline_commit':index['baseline_commit'],'accepted_baseline_jarde_cli_sha256':index['accepted_jarde_cli_sha256'],
      'jarde_cli':str(jarde),'jarde_cli_sha256':sha(jarde),'jarde_cli_is_accepted_baseline':sha(jarde)==index['accepted_jarde_cli_sha256'],
      'reference_jadx':str(jadx),'reference_jadx_sha256':sha(jadx),'jadx_version_stdout':jversion.stdout.decode(errors='replace'),
      'jadx_lib_directory':str(libdir),'jadx_lib_count':len(libs),'jadx_libs':[{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)} for p in libs],
      'jdk_legs':{leg:{'home':str(v['home']),'javac_sha256':sha(v['home']/'bin/javac'),'java_sha256':sha(v['home']/'bin/java'),'javap_sha256':sha(v['home']/'bin/javap'),'release':v['release']} for leg,v in JDKS.items()},
      'probe_source':str(shared_probe),'probe_sha256':probe_sha,'frozen_inputs':str(frozen),'frozen_input_manifest_sha256':sha(frozen/'frozen-input-manifest.json'),
      'runner_source':str(runner_snapshot),'runner_source_sha256':runner_snapshot_sha256,
      'cli_label':cli_label,'cli_sha256':sha(jarde),'baseline_cli_sha256':sha(jarde) if cli_label=='baseline' else None,
      'candidate_cli_sha256':sha(jarde) if cli_label=='candidate' else None,
      'input_filter':'reused-existing-jar' if only_reused else 'all-frozen-inputs',
      'commands':commands,'cases':results}
    manifest['files']=[{'path':str(p.relative_to(out)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(out.rglob('*')) if p.is_file() and p.name!='manifest.json']
    (out/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    summary=out/'summary.md'
    original_ok=sum(c['flavors']['original'].get('compile_exit')==0 for c in results)
    jadx_ok=sum(c['flavors']['jadx'].get('compile_exit')==0 for c in results)
    cli_ok=sum(c['flavors'][cli_label].get('compile_exit')==0 for c in results)
    matrix_modes=sorted({(c['leg'],c['debug']) for c in results}); family_count=len({c['name'] for c in results})
    summary.write_text(f'''# Frozen {cli_label} replay summary\n\nFrozen inputs: {len(results)} selected input cases across {family_count} source families and {len(matrix_modes)} JDK/debug modes. Original full-class javac succeeded for {original_ok}/{len(results)}, JADX for {jadx_ok}/{len(results)}, and CLI flavor `{cli_label}` for {cli_ok}/{len(results)}. Counts are whole-class compilation only; probe exits, behavior assertions, reflection identity and all diagnostics remain separately recorded in `manifest.json`.\n\nJarde CLI label: `{cli_label}`; SHA-256: `{sha(jarde)}`. Probe SHA-256: `{probe_sha}`. Executed runner snapshot: `{runner_snapshot}`; SHA-256: `{runner_snapshot_sha256}`. All build legs use the input-specific release/debug flags, explicit empty classpath/sourcepath, fresh classes directories, and runtime `-Xverify:all` with only those classes. Existing CallHold/ExceptionHold/BoundOverload jars are byte-for-byte reused and indexed by their source paths and hashes.\n\nThis is a frozen-input replay. No class is omitted after a compile failure. The per-case source, jar, JADX tree, Jarde source, Probe.java, javac/probe stdout/stderr and hashes are retained.\n''')
    manifest['files']=[{'path':str(p.relative_to(out)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(out.rglob('*')) if p.is_file() and p.name!='manifest.json']
    (out/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'out':str(out),'cli_label':cli_label,'cli_sha256':sha(jarde),'cases':len(results),'matrix_modes':len(matrix_modes),'families':family_count,'original_compile':original_ok,'jadx_compile':jadx_ok,'cli_compile':cli_ok,'probe':probe_sha},ensure_ascii=False))

def main():
    p=argparse.ArgumentParser()
    p.add_argument('--out',type=Path,required=True)
    p.add_argument('--frozen-inputs',type=Path,required=True)
    p.add_argument('--cli',type=Path,required=True)
    p.add_argument('--cli-label',choices=('baseline','candidate'),required=True,help='record the supplied CLI under this explicit flavor name')
    p.add_argument('--only-reused',action='store_true',help='replay only the already-frozen historical jar inputs')
    p.add_argument('--select',nargs='*',default=None,help='optional fixture names for a scoped smoke replay')
    p.add_argument('--leg',choices=JDKS.keys(),default=None,help='optional JDK leg for a scoped smoke replay')
    p.add_argument('--debug-mode',choices=DEBUGS.keys(),default=None,help='optional debug mode for a scoped smoke replay')
    p.add_argument('--freeze-inputs',action='store_true',help='create frozen jars from source index only when the frozen-input directory is empty')
    a=p.parse_args()
    source_index=load_source_index()
    if a.freeze_inputs:
      freeze_inputs(a.frozen_inputs.resolve(),source_index,[])
    fm=a.frozen_inputs.resolve()/'frozen-input-manifest.json'
    if not fm.is_file(): raise SystemExit('frozen inputs missing; first run with --freeze-inputs')
    build_and_replay(a.out.resolve(),a.frozen_inputs.resolve(),a.cli,source_index,a.cli_label,a.only_reused,a.select,a.leg,a.debug_mode)
if __name__=='__main__': main()
