// Verifier-valid negative family (CF-15 Slice B). javac 23.0.1 `--release 8 -g:none`.
// Every original compiles and runs (main below), yet each marked call must stay REFUSED by the
// argument conversion dispatch: the presented type has no java.lang closed-set edge to the
// required one, so the recovery keeps quoting its refusal instead of inventing a conversion.
// The recovered text of this class is intentionally NOT compilable; its assertions are refusal
// retention, while the original class's own runtime is pinned by C2WNRunner.
public class C2WN {
    static class MyFailure extends Exception {
        MyFailure(String m) { super(m); }
    }

    static class MyError extends Error {
        MyError(String m) { super(m); }
    }

    // N1 (table-out user class): MyFailure -> Throwable is a real runtime ancestor relation but
    // stays outside the java.lang closed set. Refused until the resolution-environment upgrade
    // path exists.
    static String userWrap(int mode) {
        try {
            if (mode == 0) throw new MyFailure("user");
            return "fine";
        } catch (MyFailure e) {
            RuntimeException wrapped = new RuntimeException("w:" + e.getMessage(), e);
            throw wrapped;
        }
    }

    // N1b (table-out user class at a user method): MyError -> Throwable for `log(Throwable)`.
    static String userForward(int mode) {
        try {
            if (mode == 0) throw new MyError("err");
            return "fine";
        } catch (MyError e) {
            log(e);
            return "logged";
        }
    }

    // N2 (outside the java.lang closed set): java.io.IOException is a real Throwable but not a
    // java.lang entry, so the wrap stays refused in this slice (upgrade path, not this change).
    static String ioWrap(int mode) {
        try {
            if (mode == 0) throw new java.io.IOException("io");
            return "fine";
        } catch (java.io.IOException e) {
            RuntimeException wrapped = new RuntimeException("w:" + e.getMessage(), e);
            throw wrapped;
        }
    }

    // N5 (position-0 mismatch, user type): the one-argument constructor's only slot requires
    // java.lang.Throwable and receives MyFailure. The refusal keys on the descriptor slot, not
    // on the two-argument constructor's message position.
    static String causeOnlyUser(int mode) {
        try {
            if (mode == 0) throw new MyFailure("cause");
            return "fine";
        } catch (MyFailure e) {
            RuntimeException wrapped = new RuntimeException(e);
            throw wrapped;
        }
    }

    // N4 (array shape): arrays stay invariant on their component, so neither the array closure
    // nor the java.lang throwable table may widen them. The presented array arrives as a
    // parameter, so the refusal is the invocation's own, not a local-declaration cascade.
    static String arrayWrap(IllegalStateException[] held, int mode) {
        try {
            if (mode == 0) throw new IllegalStateException("arr");
            return "fine";
        } catch (IllegalStateException e) {
            return "held:" + allMessages(held) + ":" + e.getMessage();
        }
    }

    static String userArrayWrap(MyFailure[] held, int mode) {
        try {
            if (mode == 0) throw new MyFailure("uarr");
            return "fine";
        } catch (MyFailure e) {
            return "held:" + allMessages(held) + ":" + e.getMessage();
        }
    }

    static String allMessages(Throwable[] causes) {
        return String.valueOf(causes.length);
    }

    // N6 (multi-candidate overload): the original source cast narrows the call to
    // `pick(java.lang.Throwable)`; the descriptor confirms it. The recovery must spell the
    // required type so the recompiled source selects the same overload and not
    // `pick(IllegalStateException)`. Before the java.lang table exists this call is refused.
    static String overloadNarrow(int mode) {
        try {
            if (mode == 0) throw new IllegalStateException("ovl");
            return "fine";
        } catch (IllegalStateException e) {
            pick((Throwable) e);
            return "picked";
        }
    }

    static void pick(Throwable t) {
        System.out.println("pick-T:" + t.getMessage());
    }

    static void pick(IllegalStateException e) {
        System.out.println("pick-ISE:" + e.getMessage());
    }

    // Companion mix: a raw-type call boxes the boolean, so the argument presents java.lang.Boolean
    // into a java.lang.Object slot (the existing Object answer, untouched by this slice), and the
    // same body widens an IllegalStateException into `log0(Throwable)` once the table answers.
    static String rawMix(int mode) {
        try {
            if (mode == 0) throw new IllegalStateException("raw");
            return "fine";
        } catch (IllegalStateException e) {
            java.util.ArrayList raw = new java.util.ArrayList();
            raw.add(Boolean.valueOf(true));
            return "raw:" + raw.size() + ":" + log0(e);
        }
    }

    static String log0(Throwable t) {
        return "log0:" + (t instanceof IllegalStateException);
    }

    static void log(Throwable cause) {
        System.out.println("log:" + cause.getMessage());
    }

    public static void main(String[] args) {
        try { userWrap(0); } catch (RuntimeException e) {
            System.out.println("userWrap:" + e.getMessage() + "/" + (e.getCause() instanceof MyFailure));
        }
        System.out.println("userWrap:" + userWrap(1));
        try { System.out.println("userForward:" + userForward(0)); } catch (RuntimeException e) {
            System.out.println("userForward:threw:" + e.getMessage());
        }
        System.out.println("userForward:" + userForward(1));
        try { ioWrap(0); } catch (RuntimeException e) {
            System.out.println("ioWrap:" + e.getMessage() + "/" + (e.getCause() instanceof java.io.IOException));
        }
        System.out.println("ioWrap:" + ioWrap(1));
        try { causeOnlyUser(0); } catch (RuntimeException e) {
            System.out.println("causeOnlyUser:" + (e.getCause() instanceof MyFailure));
        }
        System.out.println("causeOnlyUser:" + causeOnlyUser(1));
        try {
            System.out.println("arrayWrap:" + arrayWrap(new IllegalStateException[] { new IllegalStateException("a") }, 0));
        } catch (RuntimeException e) {
            System.out.println("arrayWrap:threw:" + e.getMessage());
        }
        System.out.println("arrayWrap:" + arrayWrap(new IllegalStateException[] { new IllegalStateException("b") }, 1));
        try {
            System.out.println("userArrayWrap:" + userArrayWrap(new MyFailure[] { new MyFailure("c") }, 0));
        } catch (RuntimeException e) {
            System.out.println("userArrayWrap:threw:" + e.getMessage());
        }
        System.out.println("userArrayWrap:" + userArrayWrap(new MyFailure[] { new MyFailure("d") }, 1));
        try { System.out.println("overloadNarrow:" + overloadNarrow(0)); } catch (RuntimeException e) {
            System.out.println("overloadNarrow:threw:" + e.getMessage());
        }
        System.out.println("overloadNarrow:" + overloadNarrow(1));
        try { System.out.println("rawMix:" + rawMix(0)); } catch (RuntimeException e) {
            System.out.println("rawMix:threw:" + e.getMessage());
        }
        System.out.println("rawMix:" + rawMix(1));
    }
}
