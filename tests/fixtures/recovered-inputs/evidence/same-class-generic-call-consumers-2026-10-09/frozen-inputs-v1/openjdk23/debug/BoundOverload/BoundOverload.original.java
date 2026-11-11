public class BoundOverload<T extends Number & Comparable<T>> {
 public void pick(Number value) { System.out.println("number"); }
 public void pick(Comparable<T> value) { System.out.println("comparable"); }
 public void relay(T value) { pick((Number)value); }
}
