import java.util.function.ToIntFunction;

public class CapturedLambdaAdaptationProbe {
 public static int producePrefix(){return 31;}
 public static ToIntFunction<String> captured(){
  int prefix=producePrefix();
  return value -> LambdaCaptureAdaptationSupport.pickWithPrefix(prefix,value);
 }
}
