package defpackage;

/* JADX INFO: loaded from: SharedFinallyCall.class */
public class SharedFinallyCall {
    private static int cleanupCount;

    private static void cleanup() {
        cleanupCount++;
    }

    public static String handled(boolean z) {
        cleanupCount = 0;
        try {
            if (z) {
                throw new IllegalArgumentException("arg");
            }
            cleanup();
            return "normal";
        } catch (IllegalArgumentException e) {
            return "caught";
        } finally {
            cleanup();
        }
    }

    public static int count() {
        return cleanupCount;
    }
}
