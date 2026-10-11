// jarde: presentation of `cf11/ForConditionRefusals` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class ForConditionRefusals extends java.lang.Object {
    static int counter;

    static int limit;

    static int limitCalls;

    public ForConditionRefusals() {
        // @method <init>()V
        // @declaration a constructor of `cf11.ForConditionRefusals`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void tick() {
        // @method tick()V
        // @declaration a static method of `cf11.ForConditionRefusals`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        cf11.ForConditionRefusals.counter = cf11.ForConditionRefusals.counter + 1;
        return;
    }

    static int limit() {
        // @method limit()I
        // @declaration a static method of `cf11.ForConditionRefusals`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        cf11.ForConditionRefusals.limitCalls = cf11.ForConditionRefusals.limitCalls + 1;
        return cf11.ForConditionRefusals.limit;
    }

    static int sumWithLeadingTickAndBreak() {
        // jarde: not recovered: the recovery run for `sumWithLeadingTickAndBreak()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method sumWithLeadingTickAndBreak()I
        // @declaration a static method of `cf11.ForConditionRefusals`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 14 17 27
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static int sumWithAssignedCallBound() {
        // @method sumWithAssignedCallBound()I
        // @declaration a static method of `cf11.ForConditionRefusals`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int bound;
        int i;
        int sum;
        bound = 0;
        i = 0;
        sum = 0;
        while (i < (bound = limit())) {
            sum = sum + i;
            i = i + 1;
        }
        return sum + bound;
    }
}
