package cf11;

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
