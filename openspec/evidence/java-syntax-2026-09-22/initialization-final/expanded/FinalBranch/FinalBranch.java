public class FinalBranch {
  public static final int VALUE;
  static { if (FinalSupport.flag()) VALUE = 7; else VALUE = 9; }
  public static int result() { return VALUE; }
}
