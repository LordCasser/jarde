import java.util.function.Function;
import java.util.function.IntUnaryOperator;
import java.util.function.Supplier;
import java.util.function.ToIntBiFunction;
import java.util.function.ToIntFunction;

public class LambdaAdaptationRunner {
 static void run(String name,java.util.concurrent.Callable<Object> call){try{System.out.println(name+":"+call.call());}catch(Throwable error){System.out.println(name+":"+error.getClass().getName());}}
 static void wider(String name,Object value){LambdaAdaptationSupport.calls=0;ToIntFunction f=LambdaAdaptationProbe.wider();try{System.out.println(name+":"+f.applyAsInt(value)+":calls="+LambdaAdaptationSupport.calls);}catch(Throwable error){System.out.println(name+":"+error.getClass().getName()+":calls="+LambdaAdaptationSupport.calls);}}
 static void generic(String name,Object value,boolean typed){LambdaAdaptationSupport.current=value;Supplier raw=LambdaAdaptationProbe.genericString();try{if(typed){String result=(String)raw.get();System.out.println(name+":"+result);}else{Object result=raw.get();System.out.println(name+":"+result.getClass().getName()+":"+result);}}catch(Throwable error){System.out.println(name+":"+error.getClass().getName());}}
 public static void main(String[]args){
  ToIntFunction<String> strings=LambdaAdaptationProbe.strings();
  run("string",()->strings.applyAsInt("text"));
  run("null",()->strings.applyAsInt(null));
  run("wrong",()->((ToIntFunction)strings).applyAsInt(Integer.valueOf(7)));
  wider("widerString","text");
  wider("widerNull",null);
  wider("widerWrong",Integer.valueOf(7));
  ToIntFunction<String[]> arrays=LambdaAdaptationProbe.arrays();
  run("array",()->arrays.applyAsInt(new String[]{"text"}));
  run("arrayNull",()->arrays.applyAsInt(null));
  run("arrayWrong",()->((ToIntFunction)arrays).applyAsInt(new Object[]{"text"}));
  ToIntBiFunction<LambdaAdaptationSupport,String> unbound=LambdaAdaptationProbe.unbound();
  run("unbound",()->unbound.applyAsInt(new LambdaAdaptationSupport(),"text"));
  run("unboundNull",()->unbound.applyAsInt(new LambdaAdaptationSupport(),null));
  run("unboundWrong",()->((ToIntBiFunction)unbound).applyAsInt(new LambdaAdaptationSupport(),Integer.valueOf(7)));
  Function<String,LambdaAdaptationBox> constructor=LambdaAdaptationProbe.constructor();
  run("constructor",()->constructor.apply("text").value());
  IntUnaryOperator primitive=LambdaAdaptationProbe.primitive();
  run("primitive",()->primitive.applyAsInt(7));
  generic("genericRawString","text",false);
  generic("genericTypedString","text",true);
  generic("genericRawInteger",Integer.valueOf(7),false);
  generic("genericTypedInteger",Integer.valueOf(7),true);
 }
}
