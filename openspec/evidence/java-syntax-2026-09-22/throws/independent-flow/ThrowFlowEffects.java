public class ThrowFlowEffects {
 public static int calls;public static int mode;
 public static final RuntimeException FAIL=new IllegalStateException("producer");
 public static final RuntimeException VALUE=new IllegalArgumentException("value");
 public static String argument(String text){calls=calls*10+1;if(mode==1)throw FAIL;return text;}
 public static RuntimeException problem(){calls=calls*10+3;if(mode==3)throw FAIL;return VALUE;}
 public static RuntimeException wrapper(RuntimeException x){calls=calls*10+4;if(mode==4)throw FAIL;return x;}
}
