package em07;
public class Runner {
  public static void main(String[] args) {
    Assignment assignment = new Assignment();
    long first = assignment.setValue(4294967297L);
    System.out.println(first + ":" + assignment.getValue());
    long second = assignment.setValue(-1L);
    System.out.println(second + ":" + assignment.getValue());
  }
}
