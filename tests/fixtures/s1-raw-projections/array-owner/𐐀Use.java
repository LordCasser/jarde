package p;
public class 𐐀Use {
  static String trace = "";
  byte[] bytes;
  static byte mark(int value) { trace += "eval:" + value + ";"; return (byte)value; }
  public 𐐀Use() { super(); this.bytes = new byte[]{ mark(10), mark(20) }; trace += "body:noarg;"; }
  public 𐐀Use(int marker) { super(); this.bytes = new byte[]{ mark(10), mark(20) }; trace += "body:int:" + marker + ";"; }
}
