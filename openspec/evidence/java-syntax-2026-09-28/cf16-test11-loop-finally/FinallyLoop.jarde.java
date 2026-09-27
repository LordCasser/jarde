// jarde: presentation of `FinallyLoop` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class FinallyLoop extends java.lang.Object {
    private int count;

    public static boolean fail;

    public FinallyLoop() {
        // @method <init>()V
        // @declaration a constructor of `FinallyLoop`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `test(Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): method body or no-body declaration has no complete source proof
    public void test(java.util.List list) {
        // jarde: not recovered: the recovery run for `test(Ljava/util/List;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test(Ljava/util/List;)V
        // @declaration an instance method of `FinallyLoop`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5 10 11 12 17 20 21 26 27 28 29 32 35 38 40 41 46 48 50 55 58 60 65 67 68 70 73 76 78 79
        // the graph is not reducible over 2 block(s) [48, 58]: a loop is entered at more than its header or two loops cross, which no Java structure spells
    }

    private void call1() {
        // @method call1()V
        // @declaration an instance method of `FinallyLoop`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        this.count += 100;
        if (FinallyLoop.fail) {
            throw new java.lang.IllegalStateException("body");
        } else {
            return;
        }
    }

    private void call2(java.lang.Object item) {
        // @method call2(Ljava/lang/Object;)V
        // @declaration an instance method of `FinallyLoop`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        this.count++;
        return;
    }

    public int count() {
        // @method count()I
        // @declaration an instance method of `FinallyLoop`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.count;
    }
}
