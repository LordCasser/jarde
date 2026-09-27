package em18;
public class Runner {
  public static void main(String[] args) {
    System.out.println(java.util.Arrays.toString(Arrays.strings()));
    System.out.println(java.util.Arrays.toString(Arrays.ints(2)));
    System.out.println(java.util.Arrays.toString(Arrays.postfix(2)));
    System.out.println(java.util.Arrays.toString(Arrays.selfRead()));
    System.out.println(Arrays.objectArg(new Exception()));
  }
}
