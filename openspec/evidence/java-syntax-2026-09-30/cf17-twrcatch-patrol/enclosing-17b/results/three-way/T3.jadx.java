package defpackage;

/* JADX INFO: loaded from: T3.class */
public class T3 implements AutoCloseable {
    @Override // java.lang.AutoCloseable
    public void close() {
    }

    static void touch(T3 t3) {
    }

    public static String voidNamed() throws Exception {
        try {
            T3 t3 = new T3();
            try {
                touch(t3);
                t3.close();
                return "done";
            } catch (Throwable th) {
                try {
                    t3.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            return "caught";
        }
    }

    public static String voidNamedRecover() throws Exception {
        try {
            T3 t3 = new T3();
            try {
                touch(t3);
                t3.close();
                return "done";
            } catch (Throwable th) {
                try {
                    t3.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            touch(null);
            return "done";
        }
    }

    public static void main(String[] strArr) throws Exception {
        System.out.println(voidNamed());
        System.out.println(voidNamedRecover());
    }
}
