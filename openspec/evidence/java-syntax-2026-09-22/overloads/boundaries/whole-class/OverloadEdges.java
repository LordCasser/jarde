import java.util.function.Supplier;
public class OverloadEdges {
 public static int choose(Object x){return 1;}
 public static int choose(String x){return 2;}
 public static int arr(Object x){return 3;}
 public static int arr(String[] x){return 4;}
 public static int boxed(Object x){return 5;}
 public static int boxed(Integer x){return 6;}
 public static int action(Runnable x){return 7;}
 public static int action(Supplier<String> x){return 8;}
 public static int nullObject(){return choose((Object)null);}
 public static int arrayObject(String[] x){return arr((Object)x);}
 public static int boxObject(int x){return boxed((Object)Integer.valueOf(x));}
 public static int lambdaRunnable(){return action((Runnable)() -> System.nanoTime());}
 public static int lambdaSupplier(){return action((Supplier<String>)() -> "s");}
 public static int methodRefRunnable(){return action((Runnable)System::nanoTime);}
}
