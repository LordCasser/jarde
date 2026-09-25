public class ReturnHelper {
 public static Object current;
 @SuppressWarnings("unchecked") public static <T> T value(){return (T)current;}
}
