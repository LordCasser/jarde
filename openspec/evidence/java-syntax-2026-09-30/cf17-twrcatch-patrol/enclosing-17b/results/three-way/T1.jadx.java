package defpackage;

/* JADX INFO: loaded from: T1.class */
public class T1 implements AutoCloseable {
    @Override // java.lang.AutoCloseable
    public void close() {
    }

    public static String twrVoid() throws Exception {
        T1 t1 = new T1();
        try {
            t1.hashCode();
            t1.close();
            return "done";
        } catch (Throwable th) {
            try {
                t1.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String twrPop() throws Exception {
        T1 t1 = new T1();
        try {
            t1.toString();
            t1.close();
            return "done";
        } catch (Throwable th) {
            try {
                t1.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String twrPopNamed() throws Exception {
        try {
            T1 t1 = new T1();
            try {
                t1.toString();
                t1.close();
                return "done";
            } catch (Throwable th) {
                try {
                    t1.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            return "caught";
        }
    }

    public static String twrVoidNamed() throws Exception {
        try {
            T1 t1 = new T1();
            try {
                t1.hashCode();
                t1.close();
                return "done";
            } catch (Throwable th) {
                try {
                    t1.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            return "caught";
        }
    }

    public static void main(String[] strArr) throws Exception {
        System.out.println(twrVoid());
        System.out.println(twrPop());
        System.out.println(twrPopNamed());
        System.out.println(twrVoidNamed());
    }
}
