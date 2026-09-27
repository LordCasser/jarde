package em17;
public class Shapes {
  private char[] payload;
  public Shapes(byte[] bytes) {
    char[] chars = toChars(bytes);
    this.payload = new char[chars.length];
    System.arraycopy(chars, 0, this.payload, 0, bytes.length);
  }
  private static char[] toChars(byte[] bytes) { return new char[bytes.length]; }
  public int payloadLength() { return payload.length; }
  public static long[][] longRows(int n) { return new long[n][]; }
  public static String[][] stringRows(int n) { return new String[n][]; }
  public static int[][][] deep(int n) { return new int[n][][]; }
  public static int[][] full(int a, int b) { return new int[a][b]; }
  public static int[][] literal(int a, int b) { return new int[][]{new int[]{1}, new int[]{a, b}, new int[0]}; }
  public static Object[] wrapped(byte[] bytes) { return new Object[]{bytes}; }
}
