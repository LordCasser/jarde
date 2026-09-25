public class InlineOrderEffects {
 public static int trace,mode;
 public static InlineHolder shared;
 public static final RuntimeException FAIL=new IllegalStateException("chosen");
 static void step(int n){trace=trace*10+n;if(mode==n)throw FAIL;}
 public static int value(){step(1);shared.value=99;return 5;}
 public static int take(int x){step(2);return x+1;}
 public static void accept(int x){step(2);}
 public static InlineHolder make(){step(3);return shared;}
}
