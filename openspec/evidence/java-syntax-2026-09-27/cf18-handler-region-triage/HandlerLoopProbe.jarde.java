// jarde: presentation of `HandlerLoopProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class HandlerLoopProbe extends java.lang.Object {
    private static int effects;

    public HandlerLoopProbe() {
        // @method <init>()V
        // @declaration a constructor of `HandlerLoopProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int work(int arg0) {
        // @method work(I)I
        // @declaration a static method of `HandlerLoopProbe`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 == 0) {
            throw new java.lang.NumberFormatException("zero");
        } else if (arg0 == 2) {
            throw new java.lang.IllegalStateException("two");
    } else {
            return arg0;
    }
    }

    private static int run() {
        // jarde: not recovered: the recovery run for `run()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method run()I
        // @declaration a static method of `HandlerLoopProbe`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 4 5 6 9 10 11 14 15 16 19 20 23 25 26 29 32 35 38 39 42 44 45 48 51 54 57 58
        // the graph is not reducible over 4 block(s) [4, 9, 32, 51]: a loop is entered at more than its header or two loops cross, which no Java structure spells
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `HandlerLoopProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(run()).append(":").append(HandlerLoopProbe.effects).toString());
        return;
    }
}
