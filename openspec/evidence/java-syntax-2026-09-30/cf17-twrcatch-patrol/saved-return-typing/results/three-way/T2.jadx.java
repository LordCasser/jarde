package defpackage;

/* JADX INFO: loaded from: T2.class */
public class T2 implements AutoCloseable {
    @Override // java.lang.AutoCloseable
    public void close() {
    }

    static void touch(T2 t2) {
    }

    public static String voidBody() throws Exception {
        T2 t2 = new T2();
        try {
            touch(t2);
            t2.close();
            return "done";
        } catch (Throwable th) {
            try {
                t2.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String voidBodyReturnInside() throws Exception {
        T2 t2 = new T2();
        try {
            touch(t2);
            t2.close();
            return "in";
        } catch (Throwable th) {
            try {
                t2.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String popBody() throws Exception {
        T2 t2 = new T2();
        try {
            t2.toString();
            t2.close();
            return "done";
        } catch (Throwable th) {
            try {
                t2.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String popBodyVoidTouch() throws Exception {
        T2 t2 = new T2();
        try {
            t2.hashCode();
            touch(t2);
            t2.close();
            return "done";
        } catch (Throwable th) {
            try {
                t2.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static void main(String[] strArr) throws Exception {
        System.out.println(voidBody());
        System.out.println(voidBodyReturnInside());
        System.out.println(popBody());
        System.out.println(popBodyVoidTouch());
    }
}
