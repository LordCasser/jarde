package defpackage;

/* JADX INFO: loaded from: P3.class */
public class P3 implements AutoCloseable {
    static StringBuilder log = new StringBuilder();

    static void touch(P3 p3) {
        log.append("t");
    }

    static void boom() {
        if (log.length() > 1000) {
            throw new IllegalStateException("x");
        }
    }

    @Override // java.lang.AutoCloseable
    public void close() {
        log.append("[c]");
    }

    public static String voidBodyCatch() throws Exception {
        try {
            P3 p3 = new P3();
            try {
                touch(p3);
                p3.close();
            } catch (Throwable th) {
                try {
                    p3.close();
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

    public static String voidBodyBranch() throws Exception {
        P3 p3 = new P3();
        try {
            touch(p3);
            boom();
            p3.close();
            return log.toString();
        } catch (Throwable th) {
            try {
                p3.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String voidBodySoloFin() throws Exception {
        try {
            P3 p3 = new P3();
            try {
                touch(p3);
                p3.close();
                log.append("f");
                return log.toString();
            } catch (Throwable th) {
                try {
                    p3.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (Throwable th3) {
            log.append("f");
            throw th3;
        }
    }

    public static void main(String[] strArr) throws Exception {
        System.out.println(voidBodyCatch());
        System.out.println(voidBodyBranch());
        System.out.println(voidBodySoloFin());
    }
}
