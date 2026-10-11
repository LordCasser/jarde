package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/CF11SlotReuseControl.class */
public final class CF11SlotReuseControl {
    public static Object afterStringLifetime(boolean replace, List values) {
        "a".length();
        Object target = "seed";
        if (replace) {
            target = values.get(0);
        }
        return target;
    }
}
