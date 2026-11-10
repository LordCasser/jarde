package p;
public class RawArrayUse {
  static Class scalar() { return 𐐀Type.class; }
  static Class literal() { return 𐐀Type[][].class; }
  static 𐐀Type[] one(int n) { return new 𐐀Type[n]; }
  static 𐐀Type[][] partial(int n) { return new 𐐀Type[n][]; }
  static 𐐀Type[][] full(int n, int m) { return new 𐐀Type[n][m]; }
  static int len(𐐀Type[] a) { return a.length; }
  static 𐐀Type take(𐐀Type[] a, int i) { return a[i]; }
  static void put(𐐀Type[] a, int i, 𐐀Type v) { a[i] = v; }
}
