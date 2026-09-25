// jarde: presentation of `FinallyStraightThrow` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class FinallyStraightThrow extends java.lang.Object {
    public static int trace;

    public static boolean failTry;

    public static boolean failCleanup;

    public static final java.lang.RuntimeException TRY_FAILURE;

    public static final java.lang.RuntimeException CLEANUP_FAILURE;

    public FinallyStraightThrow() {
        // @method <init>()V
        // @declaration a constructor of `FinallyStraightThrow`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int value() {
        // @method value()I
        // @declaration a static method of `FinallyStraightThrow`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        FinallyStraightThrow.trace = FinallyStraightThrow.trace * 10 + 1;
        if (FinallyStraightThrow.failTry) {
            throw FinallyStraightThrow.TRY_FAILURE;
        } else {
            return 41;
        }
    }

    private static void cleanup() {
        // @method cleanup()V
        // @declaration a static method of `FinallyStraightThrow`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        FinallyStraightThrow.trace = FinallyStraightThrow.trace * 10 + 2;
        if (FinallyStraightThrow.failCleanup) {
            throw FinallyStraightThrow.CLEANUP_FAILURE;
        } else {
            return;
        }
    }

    public static int run() {
        // @method run()I
        // @declaration a static method of `FinallyStraightThrow`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            int local0 = value();
            return local0;
        } finally {
            cleanup();
        }
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `FinallyStraightThrow`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        TRY_FAILURE = new java.lang.IllegalArgumentException("try");
        CLEANUP_FAILURE = new java.lang.IllegalStateException("cleanup");
    }
}
