public class OverloadRunner {
  public static void main(String[] args) {
    System.out.println("nullObject=" + OverloadEdges.nullObject());
    System.out.println("arrayObject=" + OverloadEdges.arrayObject(new String[]{"x"}));
    System.out.println("boxObject=" + OverloadEdges.boxObject(7));
    System.out.println("lambdaRunnable=" + OverloadEdges.lambdaRunnable());
    System.out.println("lambdaSupplier=" + OverloadEdges.lambdaSupplier());
    System.out.println("methodRefRunnable=" + OverloadEdges.methodRefRunnable());
  }
}
