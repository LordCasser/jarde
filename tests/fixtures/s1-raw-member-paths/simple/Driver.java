package p;
public class Driver {
  public static void main(String[] args) {
    𐐀Outer.effects = 0;
    𐐀Outer.Inner value = Use.make(new 𐐀Outer(), 7);
    System.out.println(value.value());
    System.out.println(value.getClass().getName());
    System.out.println(𐐀Outer.effects);
    try { Use.make(null, 9); } catch (NullPointerException e) { System.out.println("null"); }
    System.out.println(𐐀Outer.effects);
  }
}
