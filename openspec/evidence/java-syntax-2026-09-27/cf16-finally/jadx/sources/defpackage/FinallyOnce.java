package defpackage;

/* JADX INFO: loaded from: FinallyOnce.class */
public class FinallyOnce {
    private static int cleanupCount;

    public static String handled(boolean fail) {
        cleanupCount = 0;
        try {
            if (fail) {
                throw new IllegalArgumentException("arg");
            }
            cleanupCount++;
            return "normal";
        } catch (IllegalArgumentException error) {
            String str = "caught:" + error.getMessage();
            int i = cleanupCount;
            return str;
        } finally {
            cleanupCount++;
        }
    }

    public static void escaping() {
        cleanupCount = 0;
        try {
            throw new IllegalStateException("state");
        } catch (Throwable th) {
            cleanupCount++;
            throw th;
        }
    }

    public static int count() {
        return cleanupCount;
    }

    public static void main(String[] args) {
        System.out.println(handled(false) + ":" + count());
        System.out.println(handled(true) + ":" + count());
        try {
            escaping();
            System.out.println("missing throw");
        } catch (IllegalStateException error) {
            System.out.println(error.getMessage() + ":" + count());
        }
    }
}
