import java.util.function.ToIntFunction;

public class ArrayProbe {
 public static int pick(Object[] value){return 30;}
 public static int pick(String[] value){return 31;}
 public static ToIntFunction<String[]> strings(){return ArrayProbe::pick;}
 public static ToIntFunction<Object[]> objects(){return ArrayProbe::pick;}
}
