// jarde: presentation of `P3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class P3 extends java.lang.Object implements java.lang.AutoCloseable {
    static java.lang.StringBuilder log = new java.lang.StringBuilder();

    public P3() {
        // @method <init>()V
        // @declaration a constructor of `P3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void touch(P3 arg0) {
        // @method touch(LP3;)V
        // @declaration a static method of `P3`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        P3.log.append("t");
        return;
    }

    static void boom() {
        // @method boom()V
        // @declaration a static method of `P3`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (P3.log.length() > 1000) {
            throw new java.lang.IllegalStateException("x");
        } else {
            return;
        }
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `P3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        P3.log.append("[c]");
        return;
    }

    public static java.lang.String voidBodyCatch() throws java.lang.Exception {
        // jarde: not recovered: the recovery run for `voidBodyCatch()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method voidBodyCatch()Ljava/lang/String;
        // @declaration a static method of `P3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 19 27 33 35 38 48
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String voidBodyBranch() throws java.lang.Exception {
        // @method voidBodyBranch()Ljava/lang/String;
        // @declaration a static method of `P3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (P3 local0 = new P3()) {
            touch(local0);
            boom();
        }
        return P3.log.toString();
    }

    public static java.lang.String voidBodySoloFin() throws java.lang.Exception {
        // @method voidBodySoloFin()Ljava/lang/String;
        // @declaration a static method of `P3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (P3 local0 = new P3()) {
            touch(local0);
        } finally {
            P3.log.append("f");
        }
        return P3.log.toString();
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `P3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) voidBodyCatch());
        java.lang.System.out.println((java.lang.String) voidBodyBranch());
        java.lang.System.out.println((java.lang.String) voidBodySoloFin());
        return;
    }
}
