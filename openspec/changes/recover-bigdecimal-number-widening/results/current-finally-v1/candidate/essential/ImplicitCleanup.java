// jarde: presentation of `ImplicitCleanup` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ImplicitCleanup extends java.lang.Object {
    public static int trace;

    public static boolean throwTry;

    public static boolean throwCleanup;

    public static final java.lang.RuntimeException TRY_FAILURE;

    public static final java.lang.RuntimeException CLEANUP_FAILURE;

    public ImplicitCleanup() {
        // @method <init>()V
        // @declaration a constructor of `ImplicitCleanup`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int mark(int arg0) {
        // @method mark(I)I
        // @declaration a static method of `ImplicitCleanup`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        ImplicitCleanup.trace = ImplicitCleanup.trace * 10 + arg0;
        return arg0;
    }

    private static void cleanup() {
        // @method cleanup()V
        // @declaration a static method of `ImplicitCleanup`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        ImplicitCleanup.trace = ImplicitCleanup.trace * 10 + 9;
        if (ImplicitCleanup.throwCleanup) {
            throw ImplicitCleanup.CLEANUP_FAILURE;
        } else {
            return;
        }
    }

    public static int run() {
        // @method run()I
        // @declaration a static method of `ImplicitCleanup`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (ImplicitCleanup.throwTry) {
                mark(1);
                throw ImplicitCleanup.TRY_FAILURE;
            } else {
                int local0 = mark(2);
                return local0;
            }
        } finally {
            cleanup();
        }
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `ImplicitCleanup`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        TRY_FAILURE = new java.lang.IllegalArgumentException("try");
        CLEANUP_FAILURE = new java.lang.IllegalStateException("cleanup");
    }
}
