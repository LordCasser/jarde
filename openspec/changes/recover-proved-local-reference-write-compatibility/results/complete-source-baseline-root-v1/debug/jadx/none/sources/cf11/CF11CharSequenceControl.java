package cf11;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/CF11CharSequenceControl.class */
public final class CF11CharSequenceControl {
    public static int stringLength() {
        return "string".length();
    }

    public static int builderLength() {
        CharSequence value = new StringBuilder("builder");
        return value.length();
    }
}
