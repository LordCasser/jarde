import java.lang.reflect.Array;
public class PartialArraysRunner {
 interface Task{Object run();}
 static String shape(Object x){if(x==null)return "null";if(!x.getClass().isArray())return String.valueOf(x);int n=Array.getLength(x);return x.getClass().getName()+":"+n+(n>0&&x instanceof Object[]?":"+shape(Array.get(x,0)):"");}
 static void test(String name,int mode,Task t){PartialArrayEffects.trace=0;PartialArrayEffects.mode=mode;PartialArrays.held=null;try{System.out.println(name+":"+mode+":"+shape(t.run())+":"+PartialArrayEffects.trace);}catch(Throwable e){System.out.println(name+":"+mode+":"+e.getClass().getName()+":"+(e==PartialArrayEffects.FAIL)+":"+PartialArrayEffects.trace);}}
 public static void main(String[]args){for(final int n:new int[]{-1,0,2}){
  test("primitive"+n,0,()->PartialArrays.primitive(n));test("reference"+n,0,()->PartialArrays.reference(n));test("local"+n,0,()->PartialArrays.local(n));test("field"+n,0,()->{PartialArrays.field(n);return PartialArrays.held;});test("length"+n,0,()->PartialArrays.length(n));
 }
 for(final int a:new int[]{-1,0,2})for(final int b:new int[]{-1,0,2}){test("prefix"+a+","+b,0,()->PartialArrays.prefix(a,b));test("referencePrefix"+a+","+b,0,()->PartialArrays.referencePrefix(a,b));test("complete"+a+","+b,0,()->PartialArrays.complete(a,b));for(int m=0;m<=2;m++)test("effects"+a+","+b,m,()->PartialArrays.effects(a,b));}
 }
}
