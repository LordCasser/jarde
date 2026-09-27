// jarde: presentation of `ExceptionRegionsAudit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ExceptionRegionsAudit extends java.lang.Object {
    private static int effects;

    public ExceptionRegionsAudit() {
        // @method <init>()V
        // @declaration a constructor of `ExceptionRegionsAudit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int work(int arg0) {
        // @method work(I)I
        // @declaration a static method of `ExceptionRegionsAudit`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        ExceptionRegionsAudit.effects = ExceptionRegionsAudit.effects + 1;
        if (arg0 == 0) {
            throw new java.lang.NumberFormatException("zero");
        } else if (arg0 == 2) {
            throw new java.lang.IllegalStateException("two");
    } else {
            return arg0 * 3;
    }
    }

    private static int run() {
        // jarde: not recovered: the recovery run for `run()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method run()I
        // @declaration a static method of `ExceptionRegionsAudit`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5 6 7 8 9 10 13 14 17 20 21 22 25 28 29 30 33 34 35 38 39 42 44 45 48 49 52 55 58 59 60 63 66 69 72 73 76 78 79 82 85 88 91 92
        // the graph is not reducible over 8 block(s) [8, 13, 17, 28, 85, 58, 63, 66]: a loop is entered at more than its header or two loops cross, which no Java structure spells
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ExceptionRegionsAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(run()).append(":").append(ExceptionRegionsAudit.effects).toString());
        return;
    }
}
