// Throwable-wrap positive family (CF-15 Slice B). javac 23.0.1 `--release 8 -g:none`.
// Every invocation whose cause slot requires `java.lang.Throwable` receives a java.lang
// exception the closed-set widening answer must present with the required spelling.
public class C2W {
    // V1 = the frozen C2.alias shape: IllegalStateException -> Throwable at parameter 1.
    public static String alias(int mode) {
        try {
            if (mode == 0) throw new IllegalStateException("zero");
            return "fine";
        } catch (IllegalStateException e) {
            String m = e.getMessage();
            RuntimeException wrapped = new RuntimeException("w:" + m, e);
            throw wrapped;
        }
    }

    // V2: IllegalArgumentException -> Throwable at parameter 1.
    public static String iaeWrap(int mode) {
        try {
            if (mode == 0) throw new IllegalArgumentException("bad");
            return "fine";
        } catch (IllegalArgumentException e) {
            RuntimeException wrapped = new RuntimeException("w:" + e.getMessage(), e);
            throw wrapped;
        }
    }

    // V3: nested double wrap at one catch level. `a` exercises IllegalStateException ->
    // Throwable and `w1` exercises the second edge RuntimeException -> Throwable, so the
    // recompiled class keeps a two-link cause chain (w2 -> w1 -> ISE). (The lexically nested
    // try-in-try spelling of the same chain needs the Slice A crossing-local channel and is
    // recorded in the wrap README as out of this slice's domain.)
    public static String nested(int mode) {
        try {
            if (mode == 0) throw new IllegalStateException("inner");
            return "fine";
        } catch (IllegalStateException a) {
            RuntimeException w1 = new RuntimeException("w1", a);
            RuntimeException w2 = new RuntimeException("w2:" + w1.getMessage(), w1);
            throw w2;
        }
    }

    // V4: a user static method receives the caught exception as a java.lang.Throwable parameter.
    public static String forward(int mode) {
        try {
            if (mode == 0) throw new IllegalStateException("fwd");
            return "fine";
        } catch (IllegalStateException e) {
            log(e);
            return "logged";
        }
    }

    static void log(Throwable cause) {
        System.out.println("log:" + cause.getMessage());
    }

    // V5: several argument positions in one call. Only the middle slot is a Throwable; a wrong
    // widening of the int or char slots would not compile, so this shape also falsifies a table
    // that ignores positions.
    public static String multi(int mode) {
        try {
            if (mode == 0) throw new IllegalStateException("multi");
            return "fine";
        } catch (IllegalStateException e) {
            note(7, e, 'x');
            return "noted";
        }
    }

    static void note(int code, Throwable cause, char tag) {
        System.out.println("note:" + code + ":" + cause.getMessage() + tag);
    }

    // V6: the one-argument constructor resolves to `(Throwable)` in the original source, so
    // parameter 0 (not 1) is the widening position here.
    public static String causeOnly(int mode) {
        try {
            if (mode == 0) throw new IllegalStateException("cause");
            return "fine";
        } catch (IllegalStateException e) {
            RuntimeException wrapped = new RuntimeException(e);
            throw wrapped;
        }
    }

    // V7 (multi-candidate overload): the original source cast selects `pick(java.lang.Throwable)`;
    // the recovery must spell the required type so the recompiled source cannot retarget the call
    // to `pick(IllegalStateException)`.
    public static String overloadNarrow(int mode) {
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

    // Existing Object-target answer must keep answering first: an IllegalStateException argument
    // for an `Object` parameter is the Object branch, never the Throwable table.
    public static String objectTarget(int mode) {
        try {
            if (mode == 0) throw new IllegalStateException("obj");
            return "fine";
        } catch (IllegalStateException e) {
            return "id:" + identity(e);
        }
    }

    static Object identity(Object o) {
        return o;
    }

    public static void main(String[] args) {
        try { alias(0); } catch (RuntimeException e) {
            System.out.println("alias:" + e.getMessage() + "/" + (e.getCause() instanceof IllegalStateException));
        }
        System.out.println("alias:" + alias(1));
        try { iaeWrap(0); } catch (RuntimeException e) {
            System.out.println("iaeWrap:" + e.getMessage() + "/" + (e.getCause() instanceof IllegalArgumentException));
        }
        System.out.println("iaeWrap:" + iaeWrap(1));
        try { nested(0); } catch (RuntimeException e) {
            RuntimeException mid = (RuntimeException) e.getCause();
            System.out.println("nested:" + e.getMessage() + "/" + mid.getMessage()
                + "/" + (mid.getCause() instanceof IllegalStateException));
        }
        System.out.println("nested:" + nested(1));
        try { System.out.println("forward:" + forward(0)); } catch (RuntimeException e) {
            System.out.println("forward:threw:" + e.getMessage());
        }
        System.out.println("forward:" + forward(1));
        try { System.out.println("multi:" + multi(0)); } catch (RuntimeException e) {
            System.out.println("multi:threw:" + e.getMessage());
        }
        System.out.println("multi:" + multi(1));
        try { causeOnly(0); } catch (RuntimeException e) {
            System.out.println("causeOnly:" + (e.getCause() instanceof IllegalStateException) + ":" + e.getMessage());
        }
        System.out.println("causeOnly:" + causeOnly(1));
        try { System.out.println("overloadNarrow:" + overloadNarrow(0)); } catch (RuntimeException e) {
            System.out.println("overloadNarrow:threw:" + e.getMessage());
        }
        System.out.println("overloadNarrow:" + overloadNarrow(1));
        try { System.out.println("objectTarget:" + objectTarget(0)); } catch (RuntimeException e) {
            System.out.println("objectTarget:threw:" + e.getMessage());
        }
        System.out.println("objectTarget:" + objectTarget(1));
    }
}
