package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/CF11ParameterControl.class */
public final class CF11ParameterControl {
    public static Object replace(Object target, List values) {
        if (!values.isEmpty()) {
            target = values.get(0);
        }
        return target;
    }
}
