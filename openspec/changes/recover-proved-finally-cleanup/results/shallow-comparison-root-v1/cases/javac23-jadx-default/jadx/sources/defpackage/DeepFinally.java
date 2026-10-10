package defpackage;

/* JADX INFO: loaded from: DeepFinally.class */
public class DeepFinally {
    static int trace;

    private static void cleanup() {
        trace++;
    }

    static int run(int i) {
        while (i > 2) {
            try {
                i--;
                while (i > 1) {
                    i--;
                }
            } catch (Throwable th) {
                cleanup();
                throw th;
            }
        }
        int i2 = i;
        cleanup();
        return i2;
    }
}
