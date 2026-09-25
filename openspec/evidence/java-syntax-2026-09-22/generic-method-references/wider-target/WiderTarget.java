import java.util.function.ToIntFunction;
public class WiderTarget {
 public static int calls;
 public static int accept(Object value){calls++;return 7;}
 public static ToIntFunction<String> strings(){return WiderTarget::accept;}
}
