import java.util.function.Function;
import java.util.function.IntUnaryOperator;
import java.util.function.Supplier;
import java.util.function.ToIntBiFunction;
import java.util.function.ToIntFunction;

public class LambdaAdaptationProbe {
 public static ToIntFunction<String> strings(){return LambdaAdaptationSupport::pick;}
 public static ToIntFunction<String> wider(){return LambdaAdaptationSupport::wider;}
 public static ToIntFunction<String[]> arrays(){return LambdaAdaptationSupport::pickArray;}
 public static ToIntBiFunction<LambdaAdaptationSupport,String> unbound(){return LambdaAdaptationSupport::instancePick;}
 public static Function<String,LambdaAdaptationBox> constructor(){return LambdaAdaptationBox::new;}
 public static IntUnaryOperator primitive(){return LambdaAdaptationProbe::primitiveValue;}
 public static int primitiveValue(int value){return value+1;}
 public static Supplier<String> genericString(){return LambdaAdaptationSupport::<String>genericValue;}
}
