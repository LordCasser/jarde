public class GuardEffects {
 public static int trace,mode,held;public static Object shared=new Object();
 public static final RuntimeException FAIL=new IllegalStateException("chosen");
 static void step(int n){trace=trace*10+n;held=held*10+(Thread.holdsLock(shared)?1:0);if(mode==n)throw FAIL;}
 public static Object lock(){step(1);return mode==6?null:shared;}
 public static int value(){step(2);return 7;}
 public static void mark(){step(3);}
}
