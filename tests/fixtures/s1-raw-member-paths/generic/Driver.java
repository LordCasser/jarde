package p;
public class Driver {
  public static void main(String[] args) {
    Outer.𐐀A.effects = 0;
    Outer.𐐀A<String> outer = new Outer.𐐀A<String>();
    Outer.𐐀A<String>.Generic<Integer> value = Use.make(outer, 7);
    System.out.println(value.value());
    System.out.println(value.getClass().getName());
    System.out.println(Outer.𐐀A.effects);
    try { Use.make(null, 9); } catch (NullPointerException e) { System.out.println("null"); }
    System.out.println(Outer.𐐀A.effects);
  }
}
