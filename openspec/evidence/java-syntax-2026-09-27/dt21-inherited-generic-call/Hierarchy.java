package dt21;
class C1<A> {}
class C2<B> extends C1<B> {
  public B call() { return null; }
}
class C3<C> extends C2<C> {}
public class Hierarchy extends C3<String> {
  public String test() {
    String str = call();
    return str;
  }
}
