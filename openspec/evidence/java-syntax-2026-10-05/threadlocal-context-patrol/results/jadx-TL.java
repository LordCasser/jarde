package defpackage;

/* JADX INFO: loaded from: TL.class */
public class TL {
    static final java.lang.ThreadLocal<java.lang.String> CTX = new TL.1();
    static final java.lang.ThreadLocal<java.lang.Integer> SEQ = java.lang.ThreadLocal.withInitial(() -> {
        return 0;
    });
    static final java.lang.InheritableThreadLocal<java.lang.String> PARENT = new java.lang.InheritableThreadLocal<>();

    static java.lang.String withCtx(java.lang.String str, java.lang.Runnable runnable) {
        java.lang.String str2 = CTX.get();
        CTX.set(str);
        try {
            runnable.run();
            java.lang.ThreadLocal<java.lang.String> threadLocal = CTX;
            return CTX.get();
        } finally {
            CTX.set(str2);
        }
    }

    static int bump() {
        int iIntValue = SEQ.get().intValue() + 1;
        SEQ.set(java.lang.Integer.valueOf(iIntValue));
        return iIntValue;
    }

    public static void main(java.lang.String[] strArr) throws java.lang.Exception {
        PARENT.set("root");
        java.lang.String[] strArr2 = new java.lang.String[1];
        TL.2 r0 = new TL.2(strArr2);
        r0.start();
        r0.join();
        java.lang.System.out.println("" + CTX.get() + "/" + withCtx("job", new TL.3()) + "/" + bump() + bump() + bump() + "/" + strArr2[0]);
    }
}
