package em19;
public class Runner {
  public static void main(String[] args) {
    System.out.println(Access.at(0) + ":" + Access.at(3));
    try { Access.at(4); } catch (ArrayIndexOutOfBoundsException e) { System.out.println("AIOOBE"); }
    System.out.println(Access.dimensions(2));
    System.out.println(java.util.Arrays.toString(Access.reverseNegate(new int[]{1, 2, 3})));
  }
}
