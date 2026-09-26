// jarde: presentation of `PreciseRethrowBoundaryProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class PreciseRethrowBoundaryProbe extends java.lang.Object {
    static java.lang.String trace;

    public PreciseRethrowBoundaryProbe() {
        // @method <init>()V
        // @declaration a constructor of `PreciseRethrowBoundaryProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void log(java.lang.Exception exception) {
        // @method log(Ljava/lang/Exception;)V
        // @declaration a static method of `PreciseRethrowBoundaryProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        PreciseRethrowBoundaryProbe.trace = new java.lang.StringBuilder().append(PreciseRethrowBoundaryProbe.trace).append((java.lang.String) exception.getClass().getName()).append("/").append((java.lang.String) exception.getMessage()).append("|").toString();
        return;
    }

    static void log(java.lang.String text) {
        // @method log(Ljava/lang/String;)V
        // @declaration a static method of `PreciseRethrowBoundaryProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        PreciseRethrowBoundaryProbe.trace = new java.lang.StringBuilder().append(PreciseRethrowBoundaryProbe.trace).append(text).append("|").toString();
        return;
    }

    static int anyFinally(int mode) {
        // @method anyFinally(I)I
        // @declaration a static method of `PreciseRethrowBoundaryProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (mode != 0) {
            // @bytecode 4 7 8 10 13
            // BCI 24: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        } else {
            int local1 = 17;
            log("finally");
            return local1;
        }
        // @bytecode 24 25 27 30 31
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [24]
    }

    static int multiCatch(int mode) throws java.text.ParseException, java.io.IOException {
        // @method multiCatch(I)I
        // @declaration a static method of `PreciseRethrowBoundaryProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (mode == 1) {
            // @bytecode 5 8 9 11 12 15
            // BCI 34: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        } else {
            if (mode == 2) {
                // @bytecode 21 24 25 27 30
                // BCI 34: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
            } else {
                return 29;
            }
        }
        // @bytecode 34 35 36 39 40
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [34]
    }

    static int changedValue(int mode) throws java.lang.Exception {
        // @method changedValue(I)I
        // @declaration a static method of `PreciseRethrowBoundaryProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (mode != 0) {
                throw new java.io.IOException("input");
            } else {
                return 31;
            }
        } catch (java.lang.Exception e) {
            log(e);
            throw new java.lang.IllegalStateException("replacement");
        }
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `PreciseRethrowBoundaryProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        PreciseRethrowBoundaryProbe.trace = "";
    }
}
