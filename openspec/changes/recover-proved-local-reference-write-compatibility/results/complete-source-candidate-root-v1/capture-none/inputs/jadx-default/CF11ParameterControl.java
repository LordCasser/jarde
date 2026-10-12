package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/CF11ParameterControl.class */
public final class CF11ParameterControl {
    public static Object replace(Object obj, List list) {
        if (!list.isEmpty()) {
            obj = list.get(0);
        }
        return obj;
    }
}
