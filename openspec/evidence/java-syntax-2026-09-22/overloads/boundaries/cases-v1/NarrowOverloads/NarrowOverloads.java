public class NarrowOverloads {
  public static int onlyByte(byte x) { return 1; }
  public static int onlyByte(int x) { return 2; }
  public static int onlyShort(short x) { return 3; }
  public static int onlyShort(int x) { return 4; }
  public static int runByte() { return onlyByte((byte) 3); }
  public static int runShort() { return onlyShort((short) 3); }
}
