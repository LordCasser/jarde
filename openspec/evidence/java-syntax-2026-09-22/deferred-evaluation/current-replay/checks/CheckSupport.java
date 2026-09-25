public class CheckSupport {
 public int value; public static CheckSupport shared; public static int trace; public static int mode;
 public static final RuntimeException FAILURE=new IllegalStateException("marker");
 public static void mark(){trace=trace*10+2;shared.value=9;if(mode==1)throw FAILURE;}
}
