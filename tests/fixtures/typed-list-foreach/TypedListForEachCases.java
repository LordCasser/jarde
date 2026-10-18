package typedforeach;

import java.util.Iterator;
import java.util.List;

public class TypedListForEachCases {
    public static Object seen;
    public static Object after;

    public static void strings(List<String> values) {
        String current = "before";
        seen = current;
        Iterator raw = values.iterator();
        while (raw.hasNext()) {
            current = (String) raw.next();
            seen = current;
        }
        after = current;
    }

    public static void integers(List<Integer> values) {
        Integer current = null;
        seen = current;
        Iterator raw = values.iterator();
        while (raw.hasNext()) {
            current = (Integer) raw.next();
            seen = current;
        }
        after = current;
    }

    @SuppressWarnings("rawtypes")
    public static void raw(List values) {
        String current = "before";
        seen = current;
        Iterator raw = values.iterator();
        while (raw.hasNext()) {
            current = (String) raw.next();
            seen = current;
        }
        after = current;
    }

    public static void mismatch(List<String> values) {
        Integer current = null;
        seen = current;
        Iterator raw = values.iterator();
        while (raw.hasNext()) {
            current = (Integer) raw.next();
            seen = current;
        }
        after = current;
    }

    public static void declared(List<String> values) {
        for (String value : values) {
            seen = value;
        }
    }
}
