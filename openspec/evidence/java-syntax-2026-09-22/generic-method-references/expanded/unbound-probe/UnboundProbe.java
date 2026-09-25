import java.util.function.ToIntBiFunction;

public class UnboundProbe {
 public static ToIntBiFunction<UnboundHelper,String> strings(){return UnboundHelper::pick;}
 public static ToIntBiFunction<UnboundHelper,String[]> arrays(){return UnboundHelper::pick;}
}
