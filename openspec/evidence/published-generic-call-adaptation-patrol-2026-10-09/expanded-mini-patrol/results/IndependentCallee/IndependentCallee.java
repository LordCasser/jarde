public class IndependentCallee<T> {
  public <U> U id(U x) { return x; }
  public T relay(T x) { return this.<T>id(x); }
  public static void main(String[] a) { Object m=new Object(); System.out.println("behavior.marker="+(new IndependentCallee<Object>().relay(m)==m)); }
}
