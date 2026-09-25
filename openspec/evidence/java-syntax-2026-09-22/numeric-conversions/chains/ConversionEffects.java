public class ConversionEffects {
 public static int trace;
 public static int left(int x,boolean fail){trace=trace*10+1;if(fail)throw new IllegalStateException();return x;}
 public static int right(int x,boolean fail){trace=trace*10+2;if(fail)throw new IllegalArgumentException();return x;}
}
