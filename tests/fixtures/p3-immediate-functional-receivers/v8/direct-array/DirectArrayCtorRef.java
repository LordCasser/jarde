import java.util.function.IntFunction;
public class DirectArrayCtorRef {
  public static int[] make(int n) { return ((IntFunction<int[]>) int[]::new).apply(n); }
}
