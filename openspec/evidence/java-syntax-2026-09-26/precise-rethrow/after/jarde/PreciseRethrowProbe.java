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
        try {
            if (mode == 1) {
                throw new java.text.ParseException("parse", 3);
            } else {
                if (mode == 2) {
                    throw new java.io.IOException("io");
                } else {
                    return 23;
                }
            }
        } catch (java.lang.Exception e) {
            log(e);
            throw e;
        }
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `PreciseRethrowProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        PreciseRethrowProbe.trace = "";
    }
}
