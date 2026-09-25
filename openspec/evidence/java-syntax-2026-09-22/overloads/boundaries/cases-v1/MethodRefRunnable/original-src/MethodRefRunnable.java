public class MethodRefRunnable {
  public static int action(Runnable x) { return 7; }
  public static int action(java.util.function.Supplier<String> x) { return 8; }
  public static int run() { return action((Runnable) System::nanoTime); }
}
