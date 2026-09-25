public class BitwiseEffects {
 public static int trace;
 public static int throwing;
 public static boolean left(boolean x){trace=trace*10+1;if(throwing==1)throw new IllegalStateException("left");return x;}
 public static boolean right(boolean x){trace=trace*10+2;if(throwing==2)throw new IllegalArgumentException("right");return x;}
}
