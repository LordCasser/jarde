package cf11;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/CF11ReferenceArrayControl.class */
public final class CF11ReferenceArrayControl {
    public static int lengthAt(boolean z) {
        CharSequence[] charSequenceArr = new String[1];
        if (z) {
            charSequenceArr = new CharSequence[1];
        }
        charSequenceArr[0] = "x";
        return charSequenceArr[0].length();
    }
}
