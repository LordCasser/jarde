public class OnlyNarrow {
  public static int onlyByte(byte value) { return 1; }
  public static int onlyShort(short value) { return 3; }
  public static int runByte() { return onlyByte((byte) 3); }
  public static int runShort() { return onlyShort((short) 3); }
}
