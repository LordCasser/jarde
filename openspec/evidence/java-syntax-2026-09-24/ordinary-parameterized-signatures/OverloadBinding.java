import java.util.List;

public class OverloadBinding {
    public static List<String> identity(List<String> values) {
        return values;
    }

    public static String choose(Object value) {
        return "object";
    }

    public static String choose(CharSequence value) {
        return "char-sequence";
    }

    public static String call(List<String> values) {
        return choose(identity(values).get(0));
    }
}
