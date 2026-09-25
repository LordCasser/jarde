public class DeferredEffects {
 public static int trace;
 public static int mode;
 public static int field;
 public static int divisor;
 public static long longDivisor;
 public static OrderValue currentHolder;
 public static int[] shared;
 public static final RuntimeException FAILURE=new IllegalStateException("chosen");
 public static int value(){trace=trace*10+1;if(mode==1)throw FAILURE;return field;}
 public static void mark(){trace=trace*10+2;field=9;if(shared!=null)shared[0]=8;if(currentHolder!=null)currentHolder.value=9;if(mode==2)throw FAILURE;}
 public static int left(){trace=trace*10+3;return field;}
 public static int right(){trace=trace*10+4;return field;}
}
