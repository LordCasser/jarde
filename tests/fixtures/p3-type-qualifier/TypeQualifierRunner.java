public class TypeQualifierRunner {
 static void reset(){arg0.value=3;arg0_2.value=7;}
 static String otherValue(ShadowOther other){return other==null?"null":String.valueOf(other.value);}
 static void operation(String label,ShadowOther other,java.util.concurrent.Callable<Object> call){
  reset();
  try{System.out.println(label+":"+call.call()+":"+arg0.value+":"+arg0_2.value+":"+otherValue(other));}
  catch(Throwable error){System.out.println(label+":"+error.getClass().getName()+":"+arg0.value+":"+arg0_2.value+":"+otherValue(other));}
 }
 static void run(String label,ShadowOther other){
  operation(label+":own",other,()->TypeQualifierProbe.ownCall());
  operation(label+":invoke",other,()->TypeQualifierProbe.invoke(other));
  operation(label+":read",other,()->TypeQualifierProbe.read(other));
  operation(label+":write",other,()->{TypeQualifierProbe.write(other,9);return "done";});
 }
 public static void main(String[]args){run("null",null);run("object",new ShadowOther());}
}
