// jarde: presentation of `TF` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class TF extends java.lang.Object implements java.lang.AutoCloseable {
    static java.lang.StringBuilder log = new java.lang.StringBuilder();

    java.lang.String name;

    TF(java.lang.String arg1) {
        // @method <init>(Ljava/lang/String;)V
        // @declaration a constructor of `TF`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.name = arg1;
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `TF`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        TF.log.append('[').append(this.name).append(']');
        return;
    }

    public static java.lang.String v1() throws java.lang.Exception {
        // @method v1()Ljava/lang/String;
        // @declaration a static method of `TF`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (TF local0 = new TF("a")) {
            try {
                TF.log.append("body");
            } finally {
                TF.log.append("mid");
            }
        }
        return TF.log.toString();
    }

    public static java.lang.String v2() throws java.lang.Exception {
        // @method v2()Ljava/lang/String;
        // @declaration a static method of `TF`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (TF local0 = new TF("a"); TF local1 = new TF("b")) {
            TF.log.append("body");
        }
        return TF.log.toString();
    }

    public static int v3() throws java.lang.Exception {
        // @method v3()I
        // @declaration a static method of `TF`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (TF local0 = new TF("a")) {
            try {
                TF.log.append("body");
                int local1 = TF.log.length();
                return local1;
            } finally {
                TF.log.append("mid");
            }
        }
    }

    public static java.lang.String nested() throws java.lang.Exception {
        // @method nested()Ljava/lang/String;
        // @declaration a static method of `TF`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (TF local0 = new TF("a")) {
            try (TF local1 = new TF("b")) {
                TF.log.append("body");
            } finally {
                TF.log.append("mid");
            }
        }
        return TF.log.toString();
    }

    public static java.lang.String solo() throws java.lang.Exception {
        // jarde: not recovered: the recovery run for `solo()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method solo()Ljava/lang/String;
        // @declaration a static method of `TF`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 26 34 40 42 54 66
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `TF`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        TF.log.setLength(0);
        java.lang.System.out.println((java.lang.String) v1());
        TF.log.setLength(0);
        java.lang.System.out.println((java.lang.String) v2());
        TF.log.setLength(0);
        java.lang.System.out.println(v3());
        TF.log.setLength(0);
        java.lang.System.out.println((java.lang.String) nested());
        TF.log.setLength(0);
        java.lang.System.out.println((java.lang.String) solo());
        return;
    }
}
