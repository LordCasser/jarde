public class CtorOverloads {
  private final int code;
  public CtorOverloads(Object x) { code = 1; }
  public CtorOverloads(String x) { code = 2; }
  public static int run() { return new CtorOverloads((Object) null).code; }
}
