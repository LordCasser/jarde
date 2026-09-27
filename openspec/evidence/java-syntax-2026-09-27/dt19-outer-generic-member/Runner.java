package dt19;
public final class Runner {
  public static void main(String[] args) {
    Outer<String> outer = new Outer<String>();
    Outer<String>.Inner inner = outer.make();
    System.out.println(inner.id("ok"));
  }
}
