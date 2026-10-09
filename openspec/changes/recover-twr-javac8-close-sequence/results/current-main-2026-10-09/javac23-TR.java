// jarde: presentation of `TR` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class TR extends java.lang.Object implements java.lang.AutoCloseable {
    private final java.lang.String name;

    public TR(java.lang.String arg1) {
        // @method <init>(Ljava/lang/String;)V
        // @declaration a constructor of `TR`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.name = arg1;
        return;
    }

    public java.lang.String use() {
        // @method use()Ljava/lang/String;
        // @declaration an instance method of `TR`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append("used:").append(this.name).toString();
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `TR`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("closed:").append(this.name).toString());
        return;
    }

    static java.lang.String one(java.lang.String arg0) {
        // @method one(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `TR`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try (TR local1 = new TR(arg0)) {
            java.lang.String local2 = local1.use();
            return local2;
        }
    }

    static java.lang.String two(java.lang.String arg0) {
        // @method two(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `TR`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try (TR local1 = new TR(arg0 + "a"); TR local2 = new TR(arg0 + "b")) {
            java.lang.String local3 = local1.use() + local2.use();
            return local3;
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `TR`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) one("p"));
        java.lang.System.out.println((java.lang.String) two("q"));
        return;
    }
}
