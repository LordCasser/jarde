package dt21;
public final class Runner {
  public static void main(String[] args) throws Exception {
    System.out.println(new Hierarchy().test() == null);
    System.out.println(Hierarchy.class.getGenericSuperclass());
  }
}
