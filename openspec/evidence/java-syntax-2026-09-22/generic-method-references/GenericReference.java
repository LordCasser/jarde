import java.util.function.ToIntFunction;
public class GenericReference {
 public static int pick(Object value){return 1;}
 public static int pick(String value){return 2;}
 public static ToIntFunction<String> strings(){return GenericReference::pick;}
 public static ToIntFunction<Object> objects(){return GenericReference::pick;}
}
