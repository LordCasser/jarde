// jarde: presentation of `cf08twolvl/TwoLevelIf` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf08twolvl;

public final class TwoLevelIf extends java.lang.Object {
    private static int calls;

    public TwoLevelIf() {
        // @method <init>()V
        // @declaration a constructor of `cf08twolvl.TwoLevelIf`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int cost(int arg0) {
        // @method cost(I)I
        // @declaration a static method of `cf08twolvl.TwoLevelIf`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        cf08twolvl.TwoLevelIf.calls = cf08twolvl.TwoLevelIf.calls + 1;
        return arg0;
    }

    static int calls() {
        // @method calls()I
        // @declaration a static method of `cf08twolvl.TwoLevelIf`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return cf08twolvl.TwoLevelIf.calls;
    }

    static void resetCalls() {
        // @method resetCalls()V
        // @declaration a static method of `cf08twolvl.TwoLevelIf`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        cf08twolvl.TwoLevelIf.calls = 0;
        return;
    }

    public static int pick(int[] arg0) {
        // jarde: not recovered: the recovery run for `pick([I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method pick([I)I
        // @declaration a static method of `cf08twolvl.TwoLevelIf`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 9 16 21 23 28 37 46 49 55 61
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }
}
