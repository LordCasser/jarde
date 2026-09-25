public class FinalSelfRead {
  public static final int VALUE;
  public static int mutable;
  static { VALUE = FinalSupport.next(); mutable = VALUE + 2; }
  public static int result() { return mutable; }
}
