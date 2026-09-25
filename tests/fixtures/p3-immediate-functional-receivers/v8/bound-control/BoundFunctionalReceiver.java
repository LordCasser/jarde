import java.util.function.IntFunction;
import java.util.function.IntUnaryOperator;
public class BoundFunctionalReceiver {
  public static int array(int n) { IntFunction<int[]> f = int[]::new; return f.apply(n).length; }
  public static int method(int n) { IntUnaryOperator f = Math::abs; return f.applyAsInt(n); }
}
