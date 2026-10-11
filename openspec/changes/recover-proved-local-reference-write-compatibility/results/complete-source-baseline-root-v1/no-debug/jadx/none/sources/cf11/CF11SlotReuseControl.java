package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/CF11SlotReuseControl.class */
public final class CF11SlotReuseControl {
    public static Object afterStringLifetime(boolean z, List list) {
        "a".length();
        return z ? list.get(0) : "seed";
    }
}
