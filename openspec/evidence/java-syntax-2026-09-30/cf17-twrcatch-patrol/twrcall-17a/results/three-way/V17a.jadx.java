package defpackage;

/* JADX INFO: loaded from: V17a.class */
public class V17a implements AutoCloseable {
    public static boolean boom = false;

    /* JADX INFO: loaded from: V17a$G17.class */
    interface G17 {
        String get();
    }

    /* JADX INFO: loaded from: V17a$G17Impl.class */
    static final class G17Impl implements G17 {
        G17Impl() {
        }

        @Override // V17a.G17
        public String get() {
            return "ig";
        }
    }

    @Override // java.lang.AutoCloseable
    public void close() {
        if (boom) {
            throw new IllegalStateException("close");
        }
    }

    static void touch(V17a v17a) {
        if (boom) {
            throw new IllegalStateException("touch");
        }
        System.out.println("touched");
    }

    static String give() {
        if (boom) {
            throw new IllegalStateException("give");
        }
        return "g";
    }

    static G17 pick() {
        if (boom) {
            throw new IllegalStateException("pick");
        }
        return new G17Impl();
    }

    public static String pureCalls() throws Exception {
        V17a v17a = new V17a();
        try {
            v17a.toString();
            v17a.hashCode();
            v17a.close();
            return "done";
        } catch (Throwable th) {
            try {
                v17a.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String mixedVoidAndCall() throws Exception {
        V17a v17a = new V17a();
        try {
            touch(v17a);
            v17a.toString();
            v17a.close();
            return "done";
        } catch (Throwable th) {
            try {
                v17a.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String callBeforeReturnInside() throws Exception {
        V17a v17a = new V17a();
        try {
            touch(v17a);
            v17a.hashCode();
            v17a.close();
            return "in";
        } catch (Throwable th) {
            try {
                v17a.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String callOnlyReturnInside() throws Exception {
        V17a v17a = new V17a();
        try {
            v17a.toString();
            v17a.close();
            return "solo";
        } catch (Throwable th) {
            try {
                v17a.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String staticCall() throws Exception {
        V17a v17a = new V17a();
        try {
            give();
            v17a.close();
            return "done";
        } catch (Throwable th) {
            try {
                v17a.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String virtualCall() throws Exception {
        V17a v17a = new V17a();
        try {
            v17a.toString();
            v17a.close();
            return "done";
        } catch (Throwable th) {
            try {
                v17a.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String interfaceCall() throws Exception {
        V17a v17a = new V17a();
        try {
            pick().get();
            v17a.close();
            return "done";
        } catch (Throwable th) {
            try {
                v17a.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String receivedLocal() throws Exception {
        V17a v17a = new V17a();
        try {
            v17a.toString();
            v17a.close();
            return "done";
        } catch (Throwable th) {
            try {
                v17a.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static void main(String[] strArr) throws Exception {
        System.out.println(pureCalls());
        System.out.println(mixedVoidAndCall());
        System.out.println(callBeforeReturnInside());
        System.out.println(callOnlyReturnInside());
        System.out.println(staticCall());
        System.out.println(virtualCall());
        System.out.println(interfaceCall());
        System.out.println(receivedLocal());
    }
}
