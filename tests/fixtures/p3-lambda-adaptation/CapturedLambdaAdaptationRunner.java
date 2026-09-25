import java.util.function.ToIntFunction;

public class CapturedLambdaAdaptationRunner {
 static void rawBad(ToIntFunction function){
  try { function.applyAsInt(Integer.valueOf(7)); }
  catch(Throwable error){System.out.println(error.getClass().getName());}
 }
 public static void main(String[] args){
  ToIntFunction<String> function=CapturedLambdaAdaptationProbe.captured();
  System.out.println(function.applyAsInt("text"));
  System.out.println(function.applyAsInt(null));
  rawBad(function);
  try { BoundNullLambdaAdaptationProbe.boundNull(); }
  catch(Throwable error){System.out.println("boundNull:"+error.getClass().getName());}
 }
}
