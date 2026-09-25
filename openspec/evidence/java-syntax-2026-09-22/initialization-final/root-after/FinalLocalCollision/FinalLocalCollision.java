public class FinalLocalCollision {
  public static final int local0;
  public static final int local0_2;
  static { int seed = FinalSupport.next(); local0 = seed + seed; local0_2 = FinalSupport.next(); }
  public static int result() { return local0 * 10 + local0_2; }
}
