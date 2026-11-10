package p;
public class Use {
  static String trace = "";
  byte[] bytes;
  static byte 𐐀mark(int value) { trace += "eval:" + value + ";"; return (byte)value; }
  public Use() { super(); this.bytes = new byte[]{ 𐐀mark(10), 𐐀mark(20) }; trace += "body:noarg;"; }
  public Use(int marker) { super(); this.bytes = new byte[]{ 𐐀mark(10), 𐐀mark(20) }; trace += "body:int:" + marker + ";"; }
}
