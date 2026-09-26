// jarde: presentation of `PreciseRethrowProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class PreciseRethrowProbe extends java.lang.Object {
    static java.lang.String trace;

    public PreciseRethrowProbe() {
        // @method <init>()V
        // @declaration a constructor of `PreciseRethrowProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void log(java.lang.Exception exception) {
        // @method log(Ljava/lang/Exception;)V
        // @declaration a static method of `PreciseRethrowProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        PreciseRethrowProbe.trace = new java.lang.StringBuilder().append(PreciseRethrowProbe.trace).append((java.lang.String) exception.getClass().getName()).append("/").append((java.lang.String) exception.getMessage()).append("|").toString();
        return;
    }

    static int precise(int mode) throws java.text.ParseException, java.io.IOException {
        // @method precise(I)I
        // @declaration a static method of `PreciseRethrowProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (mode == 1) {
            // @bytecode 5 8 9 11 12 15
            // BCI 34: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        } else {
            if (mode == 2) {
                // @bytecode 21 24 25 27 30
                // BCI 34: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
            } else {
                return 23;
            }
        }
        // @bytecode 34 35 36 39 40
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [34]
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `PreciseRethrowProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        PreciseRethrowProbe.trace = "";
    }
}
