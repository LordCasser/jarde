package cf11;

import java.util.List;

@SuppressWarnings("rawtypes")
public final class CF11SlotReuseControl {
    public static Object afterStringLifetime(boolean replace, List values) {
        {
            String first = "a";
            first.length();
        }
        Object target = "seed";
        if (replace) {
            target = values.get(0);
        }
        return target;
    }
}
