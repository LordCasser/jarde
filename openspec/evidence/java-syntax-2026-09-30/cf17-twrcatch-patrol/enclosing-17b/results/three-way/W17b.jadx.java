package defpackage;

import java.util.Arrays;

/* JADX INFO: loaded from: W17b.class */
public class W17b implements AutoCloseable {
    static boolean closeBoom = false;

    @Override // java.lang.AutoCloseable
    public void close() {
        if (closeBoom) {
            throw new IllegalStateException("close");
        }
    }

    static void touch(W17b w17b) {
    }

    static void boom() {
        throw new IllegalStateException("body");
    }

    public static String normalReturn() {
        try {
            W17b w17b = new W17b();
            try {
                touch(w17b);
                w17b.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17b.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            return "caught";
        }
    }

    public static String bodyThrows() {
        try {
            W17b w17b = new W17b();
            try {
                boom();
                w17b.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17b.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            return "caught:" + e.getMessage();
        }
    }

    public static String handlerCall() {
        try {
            W17b w17b = new W17b();
            try {
                boom();
                w17b.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17b.close();
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

    public static String closeThrows() {
        try {
            W17b w17b = new W17b();
            try {
                touch(w17b);
                w17b.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17b.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            return "caught:" + e.getMessage() + ":" + Arrays.toString(e.getSuppressed());
        }
    }

    public static String suppressedBoth() {
        try {
            W17b w17b = new W17b();
            try {
                boom();
                w17b.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17b.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            return "caught:" + e.getMessage() + ":" + Arrays.toString(e.getSuppressed());
        }
    }

    public static void main(String[] strArr) {
        System.out.println(normalReturn());
        System.out.println(bodyThrows());
        System.out.println(handlerCall());
        closeBoom = true;
        System.out.println(closeThrows());
        System.out.println(suppressedBoth());
        closeBoom = false;
    }
}
