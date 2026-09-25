package minimal;
public final class Runner {
 public static void main(String[] args) {
  System.out.println(UseSimple.make(new Outer(), 7).getClass().getName());
  try { UseSimple.make(null, 9); } catch (NullPointerException expected) { System.out.println("null"); }
 }
}
