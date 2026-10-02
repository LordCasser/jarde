// jarde: presentation of `P1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class P1 extends java.lang.Object implements java.lang.AutoCloseable {
    static java.lang.StringBuilder log = new java.lang.StringBuilder();

    public P1() {
        // @method <init>()V
        // @declaration a constructor of `P1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `P1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        P1.log.append("[c]");
        return;
    }

    public static java.lang.String soloFinally() throws java.lang.Exception {
        // jarde: not recovered: the recovery run for `soloFinally()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method soloFinally()Ljava/lang/String;
        // @declaration a static method of `P1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 24 32 38 40 52 64
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String twrCatch() throws java.lang.Exception {
        // jarde: not recovered: the recovery run for `twrCatch()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method twrCatch()Ljava/lang/String;
        // @declaration a static method of `P1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 28 38 45 53 59 61 64 74
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `P1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) soloFinally());
        java.lang.System.out.println((java.lang.String) twrCatch());
        return;
    }
}
