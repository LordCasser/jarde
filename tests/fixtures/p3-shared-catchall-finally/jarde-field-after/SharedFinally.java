// jarde: presentation of `SharedFinally` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class SharedFinally extends java.lang.Object {
    private static int cleanupCount;

    public SharedFinally() {
        // @method <init>()V
        // @declaration a constructor of `SharedFinally`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String handled(boolean arg0) {
        // @method handled(Z)Ljava/lang/String;
        // @declaration a static method of `SharedFinally`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        SharedFinally.cleanupCount = 0;
        try {
            if (arg0) {
                throw new java.lang.IllegalArgumentException("arg");
            } else {
                return "normal";
            }
        } catch (java.lang.IllegalArgumentException local1) {
            return "caught";
        } finally {
            SharedFinally.cleanupCount = SharedFinally.cleanupCount + 1;
        }
    }

    public static int count() {
        // @method count()I
        // @declaration a static method of `SharedFinally`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return SharedFinally.cleanupCount;
    }
}
