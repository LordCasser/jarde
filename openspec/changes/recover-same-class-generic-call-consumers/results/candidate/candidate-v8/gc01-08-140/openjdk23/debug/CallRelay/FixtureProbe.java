import java.lang.reflect.*;
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
}
