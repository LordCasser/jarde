// jarde: presentation of `cf08nested/NestedEffectful` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf08nested;

public final class NestedEffectful extends java.lang.Object {
    static int calls;

    public NestedEffectful() {
        // @method <init>()V
        // @declaration a constructor of `cf08nested.NestedEffectful`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int cost(int arg0) {
        // @method cost(I)I
        // @declaration a static method of `cf08nested.NestedEffectful`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        cf08nested.NestedEffectful.calls = cf08nested.NestedEffectful.calls + 1;
        return arg0;
    }

    public static int pick(int[] arg0) {
        // jarde: not recovered: the recovery run for `pick([I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method pick([I)I
        // @declaration a static method of `cf08nested.NestedEffectful`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 9 11 17 26 35 38 44 50
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }
}
