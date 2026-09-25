public class FinalFailureSupport {
 public static final RuntimeException FAILURE=new IllegalStateException("failed");
 public static int mode,calls;public static String trace="";
 public static int next(String name){calls++;trace+=name;if(mode==calls)throw FAILURE;return calls;}
}
