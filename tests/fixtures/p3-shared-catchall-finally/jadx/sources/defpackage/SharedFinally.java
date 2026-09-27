package defpackage;

/* JADX INFO: loaded from: SharedFinally.class */
public class SharedFinally {
    private static int cleanupCount;

    public static String handled(boolean z) {
        cleanupCount = 0;
        try {
            if (z) {
                throw new IllegalArgumentException("arg");
            }
            cleanupCount++;
            return "normal";
        } catch (IllegalArgumentException e) {
            int i = cleanupCount;
            return "caught";
        } finally {
            cleanupCount++;
        }
    }

    public static int count() {
        return cleanupCount;
    }
}
