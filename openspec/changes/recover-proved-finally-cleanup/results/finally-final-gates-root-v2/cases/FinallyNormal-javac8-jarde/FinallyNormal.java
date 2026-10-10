// jarde: presentation of `FinallyNormal` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class FinallyNormal extends java.lang.Object {
    public static int trace;

    public static boolean failAtOne;

    public static final java.lang.RuntimeException FAILURE;

    public FinallyNormal() {
        // @method <init>()V
        // @declaration a constructor of `FinallyNormal`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int mark(int arg0) {
        // @method mark(I)I
        // @declaration a static method of `FinallyNormal`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        FinallyNormal.trace = FinallyNormal.trace * 10 + arg0;
        if (arg0 == 1) {
            if (FinallyNormal.failAtOne) {
                throw FinallyNormal.FAILURE;
            }
        }
        return arg0;
    }

    public static int run() {
        // @method run()I
        // @declaration a static method of `FinallyNormal`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            int local0 = mark(1);
            return local0;
        } finally {
            mark(2);
        }
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `FinallyNormal`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        FAILURE = new java.lang.IllegalArgumentException("mark-1");
    }
}
