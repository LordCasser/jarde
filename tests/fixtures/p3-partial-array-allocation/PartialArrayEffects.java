public class PartialArrayEffects {
 public static int trace,mode;public static final RuntimeException FAIL=new IllegalStateException("chosen");
 public static int dim(int id,int n){trace=trace*10+id;if(mode==id)throw FAIL;return n;}
 public static String pick(Object value){return "object";}
 public static String pick(Object[] value){return "array";}
}
