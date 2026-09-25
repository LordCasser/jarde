import java.util.function.Supplier;
public class LambdaSupplier {
  public static int action(Runnable x) { return 7; }
  public static int action(Supplier<String> x) { return 8; }
  public static int run() { return action((Supplier<String>) () -> "s"); }
}
