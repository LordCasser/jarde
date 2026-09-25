public class StructuredSupport {
 public static int trace;public static int mode;public static boolean test;public static int field;
 public static final RuntimeException FAILURE=new IllegalStateException("chosen");
 public static int value(){trace=trace*10+1;if(mode==1)throw FAILURE;return field;}
 public static boolean flag(){trace=trace*10+1;if(mode==1)throw FAILURE;return test;}
 public static void mark(){trace=trace*10+2;field=9;test=false;if(mode==2)throw FAILURE;}
}
