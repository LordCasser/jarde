package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/CF11CastReadControl.class */
public final class CF11CastReadControl {
    public static int castRead(List values) {
        Object target = "seed";
        if (!values.isEmpty()) {
            target = values.get(0);
        }
        return ((String) target).length();
    }
}
