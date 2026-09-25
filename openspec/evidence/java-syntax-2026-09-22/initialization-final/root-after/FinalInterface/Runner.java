public class Runner {
  public static void main(String[] args) throws Exception {
    Class<?> sample = Class.forName(args[0]);
    System.out.println(sample.getMethod("result").invoke(null) + ":" + FinalSupport.calls);
  }
}
