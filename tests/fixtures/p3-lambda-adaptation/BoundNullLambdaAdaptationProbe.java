import java.util.function.ToIntFunction;

public class BoundNullLambdaAdaptationProbe {
 public static ToIntFunction<String> boundNull(){
  return ((LambdaCaptureAdaptationSupport)null)::instancePick;
 }
}
