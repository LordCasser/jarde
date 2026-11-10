package p;
public class Use implements Api<𐐀Value> {
  public 𐐀Value echo(𐐀Value v) { return v; }
  public static 𐐀Value cast(Object v) { return (𐐀Value)v; }
  public static boolean test(Object v) { return v instanceof 𐐀Value; }
  public static 𐐀Value[][] array(Object v) { return (𐐀Value[][])v; }
}
