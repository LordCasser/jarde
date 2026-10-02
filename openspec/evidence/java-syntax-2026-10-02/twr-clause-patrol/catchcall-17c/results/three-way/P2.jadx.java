package defpackage;

/* JADX INFO: loaded from: P2.class */
public class P2 implements AutoCloseable {
    static StringBuilder log = new StringBuilder();
    static boolean mode;

    @Override // java.lang.AutoCloseable
    public void close() {
        log.append("[c]");
    }

    public static String branchNoCatch() throws Exception {
        P2 p2 = new P2();
        try {
            if (mode) {
                throw new IllegalStateException("x");
            }
            log.append("B");
            p2.close();
            return log.toString();
        } catch (Throwable th) {
            try {
                p2.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String plainCatch() throws Exception {
        try {
            P2 p2 = new P2();
            try {
                log.append("b");
                p2.close();
            } catch (Throwable th) {
                try {
                    p2.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            log.append("E");
        }
        return log.toString();
    }

    public static void main(String[] strArr) throws Exception {
        System.out.println(branchNoCatch());
        System.out.println(plainCatch());
    }
}
