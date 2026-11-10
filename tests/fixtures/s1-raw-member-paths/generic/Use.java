package p;
public class Use {
  public static Outer.𐐀A<String>.Generic<Integer> make(
      Outer.𐐀A<String> outer, int value) {
    return outer.new Generic<Integer>(Integer.valueOf(Outer.𐐀A.mark(value)));
  }
}
