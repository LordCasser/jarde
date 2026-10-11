package cf11;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/CF11ReferenceArrayControl.class */
public final class CF11ReferenceArrayControl {
    public static int lengthAt(boolean useInterfaceArray) {
        CharSequence[] target = new String[1];
        if (useInterfaceArray) {
            target = new CharSequence[1];
        }
        target[0] = "x";
        return target[0].length();
    }
}
