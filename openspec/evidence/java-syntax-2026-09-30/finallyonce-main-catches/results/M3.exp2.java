// jarde: presentation of `M3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class M3 extends java.lang.Object {
    private static int cleanupCount;

    public M3() {
        // @method <init>()V
        // @declaration a constructor of `M3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String handled(boolean arg0) {
        // @method handled(Z)Ljava/lang/String;
        // @declaration a static method of `M3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        M3.cleanupCount = 0;
        try {
            if (arg0) {
                throw new java.lang.IllegalArgumentException("arg");
            } else {
                M3.cleanupCount = M3.cleanupCount + 1;
                return "normal";
            }
        } catch (java.lang.IllegalArgumentException local1) {
            M3.cleanupCount = M3.cleanupCount + 1;
            return "caught:" + local1.getMessage();
        } finally {
            M3.cleanupCount = M3.cleanupCount + 1;
        }
    }

    public static int count() {
        // @method count()I
        // @declaration a static method of `M3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return M3.cleanupCount;
    }

    public static void escaping() {
        // @method escaping()V
        // @declaration a static method of `M3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        M3.cleanupCount = 0;
        try {
            throw new java.lang.IllegalStateException("state");
        } catch (java.lang.Throwable local0) {
            M3.cleanupCount = M3.cleanupCount + 1;
            throw local0;
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `M3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(handled(false) + ":" + count());
        java.lang.System.out.println(handled(true) + ":" + count());
        try {
            escaping();
            java.lang.System.out.println("missing throw");
        } catch (java.lang.IllegalStateException local1) {
            java.lang.System.out.println(local1.getMessage() + ":" + count());
        }
        return;
    }
}
