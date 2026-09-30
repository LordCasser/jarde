package defpackage;

/* JADX INFO: loaded from: C2W.class */
public class C2W {
    public static String alias(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new IllegalStateException("zero");
        } catch (IllegalStateException e) {
            throw new RuntimeException("w:" + e.getMessage(), e);
        }
    }

    public static String iaeWrap(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new IllegalArgumentException("bad");
        } catch (IllegalArgumentException e) {
            throw new RuntimeException("w:" + e.getMessage(), e);
        }
    }

    public static String nested(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new IllegalStateException("inner");
        } catch (IllegalStateException e) {
            RuntimeException runtimeException = new RuntimeException("w1", e);
            throw new RuntimeException("w2:" + runtimeException.getMessage(), runtimeException);
        }
    }

    public static String forward(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new IllegalStateException("fwd");
        } catch (IllegalStateException e) {
            log(e);
            return "logged";
        }
    }

    static void log(Throwable th) {
        System.out.println("log:" + th.getMessage());
    }

    public static String multi(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new IllegalStateException("multi");
        } catch (IllegalStateException e) {
            note(7, e, 'x');
            return "noted";
        }
    }

    static void note(int i, Throwable th, char c) {
        System.out.println("note:" + i + ":" + th.getMessage() + c);
    }

    public static String causeOnly(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new IllegalStateException("cause");
        } catch (IllegalStateException e) {
            throw new RuntimeException(e);
        }
    }

    public static String overloadNarrow(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new IllegalStateException("ovl");
        } catch (IllegalStateException e) {
            pick((Throwable) e);
            return "picked";
        }
    }

    static void pick(Throwable th) {
        System.out.println("pick-T:" + th.getMessage());
    }

    static void pick(IllegalStateException illegalStateException) {
        System.out.println("pick-ISE:" + illegalStateException.getMessage());
    }

    public static String objectTarget(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new IllegalStateException("obj");
        } catch (IllegalStateException e) {
            return "id:" + identity(e);
        }
    }

    static Object identity(Object obj) {
        return obj;
    }

    public static void main(String[] strArr) {
        try {
            alias(0);
        } catch (RuntimeException e) {
            System.out.println("alias:" + e.getMessage() + "/" + (e.getCause() instanceof IllegalStateException));
        }
        System.out.println("alias:" + alias(1));
        try {
            iaeWrap(0);
        } catch (RuntimeException e2) {
            System.out.println("iaeWrap:" + e2.getMessage() + "/" + (e2.getCause() instanceof IllegalArgumentException));
        }
        System.out.println("iaeWrap:" + iaeWrap(1));
        try {
            nested(0);
        } catch (RuntimeException e3) {
            RuntimeException runtimeException = (RuntimeException) e3.getCause();
            System.out.println("nested:" + e3.getMessage() + "/" + runtimeException.getMessage() + "/" + (runtimeException.getCause() instanceof IllegalStateException));
        }
        System.out.println("nested:" + nested(1));
        try {
            System.out.println("forward:" + forward(0));
        } catch (RuntimeException e4) {
            System.out.println("forward:threw:" + e4.getMessage());
        }
        System.out.println("forward:" + forward(1));
        try {
            System.out.println("multi:" + multi(0));
        } catch (RuntimeException e5) {
            System.out.println("multi:threw:" + e5.getMessage());
        }
        System.out.println("multi:" + multi(1));
        try {
            causeOnly(0);
        } catch (RuntimeException e6) {
            System.out.println("causeOnly:" + (e6.getCause() instanceof IllegalStateException) + ":" + e6.getMessage());
        }
        System.out.println("causeOnly:" + causeOnly(1));
        try {
            System.out.println("overloadNarrow:" + overloadNarrow(0));
        } catch (RuntimeException e7) {
            System.out.println("overloadNarrow:threw:" + e7.getMessage());
        }
        System.out.println("overloadNarrow:" + overloadNarrow(1));
        try {
            System.out.println("objectTarget:" + objectTarget(0));
        } catch (RuntimeException e8) {
            System.out.println("objectTarget:threw:" + e8.getMessage());
        }
        System.out.println("objectTarget:" + objectTarget(1));
    }
}
