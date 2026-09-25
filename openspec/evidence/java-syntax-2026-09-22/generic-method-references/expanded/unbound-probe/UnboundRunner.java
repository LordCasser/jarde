import java.util.function.ToIntBiFunction;

public class UnboundRunner {
 static void run(String name,java.util.concurrent.Callable<Object> call){try{System.out.println(name+":"+call.call());}catch(Throwable error){System.out.println(name+":"+error.getClass().getName());}}
 public static void main(String[]args){
  ToIntBiFunction<UnboundHelper,String> strings=UnboundProbe.strings();
  run("string",()->strings.applyAsInt(new UnboundHelper(),"text"));
  run("null",()->strings.applyAsInt(new UnboundHelper(),null));
  run("integer",()->((ToIntBiFunction)strings).applyAsInt(new UnboundHelper(),Integer.valueOf(7)));
  ToIntBiFunction<UnboundHelper,String[]> arrays=UnboundProbe.arrays();
  run("array",()->arrays.applyAsInt(new UnboundHelper(),new String[]{"text"}));
  run("objectArray",()->((ToIntBiFunction)arrays).applyAsInt(new UnboundHelper(),new Object[]{"text"}));
 }
}
