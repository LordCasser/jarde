// jarde: presentation of `FinallyOnce` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class FinallyOnce extends java.lang.Object {
    private static int cleanupCount;

    public FinallyOnce() {
        // @method <init>()V
        // @declaration a constructor of `FinallyOnce`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String handled(boolean fail) {
        // @method handled(Z)Ljava/lang/String;
        // @declaration a static method of `FinallyOnce`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        FinallyOnce.cleanupCount = 0;
        try {
            if (fail) {
                throw new java.lang.IllegalArgumentException("arg");
            } else {
                return "normal";
            }
        } catch (java.lang.IllegalArgumentException error) {
            return "caught:" + error.getMessage();
        } finally {
            FinallyOnce.cleanupCount = FinallyOnce.cleanupCount + 1;
        }
    }

    public static void escaping() {
        // @method escaping()V
        // @declaration a static method of `FinallyOnce`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        FinallyOnce.cleanupCount = 0;
        try {
            throw new java.lang.IllegalStateException("state");
        } catch (java.lang.Throwable local0) {
            FinallyOnce.cleanupCount = FinallyOnce.cleanupCount + 1;
            throw local0;
        }
    }

    public static int count() {
        // @method count()I
        // @declaration a static method of `FinallyOnce`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return FinallyOnce.cleanupCount;
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `FinallyOnce`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(handled(false) + ":" + count());
        java.lang.System.out.println(handled(true) + ":" + count());
        try {
            escaping();
            java.lang.System.out.println("missing throw");
        } catch (java.lang.IllegalStateException error) {
            java.lang.System.out.println(error.getMessage() + ":" + count());
        }
        return;
    }
}
