// jarde: presentation of `SharedFinallyCall` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class SharedFinallyCall extends java.lang.Object {
    private static int cleanupCount;

    public SharedFinallyCall() {
        // @method <init>()V
        // @declaration a constructor of `SharedFinallyCall`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static void cleanup() {
        // @method cleanup()V
        // @declaration a static method of `SharedFinallyCall`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        SharedFinallyCall.cleanupCount = SharedFinallyCall.cleanupCount + 1;
        return;
    }

    public static java.lang.String handled(boolean arg0) {
        // @method handled(Z)Ljava/lang/String;
        // @declaration a static method of `SharedFinallyCall`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        SharedFinallyCall.cleanupCount = 0;
        try {
            if (arg0) {
                throw new java.lang.IllegalArgumentException("arg");
            } else {
                return "normal";
            }
        } catch (java.lang.IllegalArgumentException local1) {
            return "caught";
        } finally {
            cleanup();
        }
    }

    public static int count() {
        // @method count()I
        // @declaration a static method of `SharedFinallyCall`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return SharedFinallyCall.cleanupCount;
    }
}
