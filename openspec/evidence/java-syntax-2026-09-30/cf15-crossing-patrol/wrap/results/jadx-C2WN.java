package defpackage;

import java.io.IOException;
import java.util.ArrayList;

/* JADX INFO: loaded from: C2WN.class */
public class C2WN {
    /* JADX INFO: Thrown type has an unknown type hierarchy: C2WN$MyFailure */
    static String userWrap(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new MyFailure("user");
        } catch (MyFailure e) {
            throw new RuntimeException("w:" + e.getMessage(), e);
        }
    }

    /* JADX INFO: Thrown type has an unknown type hierarchy: C2WN$MyError */
    static String userForward(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new MyError("err");
        } catch (MyError e) {
            log(e);
            return "logged";
        }
    }

    static String ioWrap(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new IOException("io");
        } catch (IOException e) {
            throw new RuntimeException("w:" + e.getMessage(), e);
        }
    }

    /* JADX INFO: Thrown type has an unknown type hierarchy: C2WN$MyFailure */
    static String causeOnlyUser(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new MyFailure("cause");
        } catch (MyFailure e) {
            throw new RuntimeException((Throwable) e);
        }
    }

    static String arrayWrap(IllegalStateException[] illegalStateExceptionArr, int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new IllegalStateException("arr");
        } catch (IllegalStateException e) {
            return "held:" + allMessages(illegalStateExceptionArr) + ":" + e.getMessage();
        }
    }

    /* JADX INFO: Thrown type has an unknown type hierarchy: C2WN$MyFailure */
    static String userArrayWrap(MyFailure[] myFailureArr, int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new MyFailure("uarr");
        } catch (MyFailure e) {
            return "held:" + allMessages(myFailureArr) + ":" + e.getMessage();
        }
    }

    static String allMessages(Throwable[] thArr) {
        return String.valueOf(thArr.length);
    }

    static String overloadNarrow(int i) {
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

    static String rawMix(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new IllegalStateException("raw");
        } catch (IllegalStateException e) {
            ArrayList arrayList = new ArrayList();
            arrayList.add(true);
            return "raw:" + arrayList.size() + ":" + log0(e);
        }
    }

    static String log0(Throwable th) {
        return "log0:" + (th instanceof IllegalStateException);
    }

    static void log(Throwable th) {
        System.out.println("log:" + th.getMessage());
    }

    public static void main(String[] strArr) {
        try {
            userWrap(0);
        } catch (RuntimeException e) {
            System.out.println("userWrap:" + e.getMessage() + "/" + (e.getCause() instanceof MyFailure));
        }
        System.out.println("userWrap:" + userWrap(1));
        try {
            System.out.println("userForward:" + userForward(0));
        } catch (RuntimeException e2) {
            System.out.println("userForward:threw:" + e2.getMessage());
        }
        System.out.println("userForward:" + userForward(1));
        try {
            ioWrap(0);
        } catch (RuntimeException e3) {
            System.out.println("ioWrap:" + e3.getMessage() + "/" + (e3.getCause() instanceof IOException));
        }
        System.out.println("ioWrap:" + ioWrap(1));
        try {
            causeOnlyUser(0);
        } catch (RuntimeException e4) {
            System.out.println("causeOnlyUser:" + (e4.getCause() instanceof MyFailure));
        }
        System.out.println("causeOnlyUser:" + causeOnlyUser(1));
        try {
            System.out.println("arrayWrap:" + arrayWrap(new IllegalStateException[]{new IllegalStateException("a")}, 0));
        } catch (RuntimeException e5) {
            System.out.println("arrayWrap:threw:" + e5.getMessage());
        }
        System.out.println("arrayWrap:" + arrayWrap(new IllegalStateException[]{new IllegalStateException("b")}, 1));
        try {
            System.out.println("userArrayWrap:" + userArrayWrap(new MyFailure[]{new MyFailure("c")}, 0));
        } catch (RuntimeException e6) {
            System.out.println("userArrayWrap:threw:" + e6.getMessage());
        }
        System.out.println("userArrayWrap:" + userArrayWrap(new MyFailure[]{new MyFailure("d")}, 1));
        try {
            System.out.println("overloadNarrow:" + overloadNarrow(0));
        } catch (RuntimeException e7) {
            System.out.println("overloadNarrow:threw:" + e7.getMessage());
        }
        System.out.println("overloadNarrow:" + overloadNarrow(1));
        try {
            System.out.println("rawMix:" + rawMix(0));
        } catch (RuntimeException e8) {
            System.out.println("rawMix:threw:" + e8.getMessage());
        }
        System.out.println("rawMix:" + rawMix(1));
    }
}
