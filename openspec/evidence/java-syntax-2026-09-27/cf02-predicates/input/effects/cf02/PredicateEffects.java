package cf02;

public class PredicateEffects {
    public static int calls;

    static String probe(String value) {
        calls++;
        return value;
    }

    public static boolean named(Object value) {
        if (value == null || !(value instanceof String)) {
            return false;
        }
        return probe((String) value).length() > 0;
    }
}
