package em17;
public class Runner {
  public static void main(String[] args) {
    System.out.println(Shapes.longRows(2).length + ":" + (Shapes.longRows(2)[0] == null));
    System.out.println(Shapes.stringRows(3).length + ":" + (Shapes.stringRows(3)[0] == null));
    System.out.println(Shapes.deep(4).length + ":" + (Shapes.deep(4)[0] == null));
    System.out.println(Shapes.full(2, 3).length + ":" + Shapes.full(2, 3)[0].length);
    System.out.println(java.util.Arrays.deepToString(Shapes.literal(4, 5)));
    byte[] bytes = {1, 2};
    System.out.println(Shapes.wrapped(bytes)[0] == bytes);
    System.out.println(new Shapes(bytes).payloadLength());
  }
}
