public class ScopeSupport {
 public static int trace;public static int mode;
 public static final RuntimeException FAILURE=new IllegalStateException("chosen");
 public static int value(){trace=trace*10+1;if(mode==1)throw FAILURE;return 5;}
 public static void mark(){trace=trace*10+2;if(mode==2)throw FAILURE;}
}
