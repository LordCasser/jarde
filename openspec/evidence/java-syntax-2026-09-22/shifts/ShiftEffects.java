public class ShiftEffects {
 public static int trace;
 public static int left(boolean fail,int value){trace=trace*10+1;if(fail)throw new IllegalStateException("left");return value;}
 public static int right(boolean fail,int value){trace=trace*10+2;if(fail)throw new IllegalArgumentException("right");return value;}
}
