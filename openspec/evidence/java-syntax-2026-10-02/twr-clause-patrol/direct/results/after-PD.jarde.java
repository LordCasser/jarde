// jarde: presentation of `PD` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class PD extends java.lang.Object implements java.lang.AutoCloseable {
    static java.lang.StringBuilder log = new java.lang.StringBuilder();

    public PD() {
        // @method <init>()V
        // @declaration a constructor of `PD`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void touch(PD arg0) {
        // @method touch(LPD;)V
        // @declaration a static method of `PD`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        PD.log.append("t");
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `PD`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        PD.log.append("[c]");
        return;
    }

    public static java.lang.String soloFin() throws java.lang.Exception {
        // @method soloFin()Ljava/lang/String;
        // @declaration a static method of `PD`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (PD local0 = new PD()) {
            touch(local0);
        } finally {
            PD.log.append("f");
        }
        return PD.log.toString();
    }

    public static java.lang.String finReturn() throws java.lang.Exception {
        // jarde: not recovered: the recovery run for `finReturn()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method finReturn()Ljava/lang/String;
        // @declaration a static method of `PD`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 19 27 33 35 42
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String multiFin() throws java.lang.Exception {
        // @method multiFin()Ljava/lang/String;
        // @declaration a static method of `PD`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (PD local0 = new PD(); PD local1 = new PD()) {
            touch(local1);
        } finally {
            PD.log.append("f");
        }
        return PD.log.toString();
    }

    public static java.lang.String bodyReturn() throws java.lang.Exception {
        // @method bodyReturn()Ljava/lang/String;
        // @declaration a static method of `PD`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (PD local0 = new PD()) {
            touch(local0);
            java.lang.String local1 = PD.log.toString();
            return local1;
        } finally {
            PD.log.append("f");
        }
    }

    public static java.lang.String threeClauses() throws java.lang.Exception {
        // jarde: not recovered: the recovery run for `threeClauses()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method threeClauses()Ljava/lang/String;
        // @declaration a static method of `PD`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 19 27 33 35 47 69 81
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `PD`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) soloFin());
        java.lang.System.out.println((java.lang.String) finReturn());
        java.lang.System.out.println((java.lang.String) multiFin());
        java.lang.System.out.println((java.lang.String) bodyReturn());
        java.lang.System.out.println((java.lang.String) threeClauses());
        return;
    }
}
