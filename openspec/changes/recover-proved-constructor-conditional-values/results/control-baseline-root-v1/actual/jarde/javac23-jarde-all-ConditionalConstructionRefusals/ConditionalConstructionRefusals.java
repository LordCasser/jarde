// jarde: presentation of `cf11/ConditionalConstructionRefusals` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class ConditionalConstructionRefusals extends java.lang.Object {
    public static int events;

    public static int conditionCalls;

    public static int argumentCalls;

    public static int effectCalls;

    public static java.lang.String sharedSlot;

    public ConditionalConstructionRefusals() {
        // @method <init>()V
        // @declaration a constructor of `cf11.ConditionalConstructionRefusals`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void reset() {
        // @method reset()V
        // @declaration a static method of `cf11.ConditionalConstructionRefusals`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        cf11.ConditionalConstructionRefusals.events = 0;
        cf11.ConditionalConstructionRefusals.conditionCalls = 0;
        cf11.ConditionalConstructionRefusals.argumentCalls = 0;
        cf11.ConditionalConstructionRefusals.effectCalls = 0;
        cf11.ConditionalConstructionRefusals.sharedSlot = null;
        return;
    }

    private static boolean condition(boolean selected, boolean fail) {
        // @method condition(ZZ)Z
        // @declaration a static method of `cf11.ConditionalConstructionRefusals`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        cf11.ConditionalConstructionRefusals.events = cf11.ConditionalConstructionRefusals.events * 10 + 1;
        cf11.ConditionalConstructionRefusals.conditionCalls = cf11.ConditionalConstructionRefusals.conditionCalls + 1;
        if (fail) {
            throw new java.lang.IllegalStateException("condition");
        } else {
            return selected;
        }
    }

    private static java.lang.String argument(java.lang.String value, boolean fail) {
        // @method argument(Ljava/lang/String;Z)Ljava/lang/String;
        // @declaration a static method of `cf11.ConditionalConstructionRefusals`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        cf11.ConditionalConstructionRefusals.events = cf11.ConditionalConstructionRefusals.events * 10 + 2;
        cf11.ConditionalConstructionRefusals.argumentCalls = cf11.ConditionalConstructionRefusals.argumentCalls + 1;
        if (fail) {
            throw new java.lang.IllegalArgumentException("argument");
        } else {
            return value;
        }
    }

    private static void independentEffect() {
        // @method independentEffect()V
        // @declaration a static method of `cf11.ConditionalConstructionRefusals`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        cf11.ConditionalConstructionRefusals.events = cf11.ConditionalConstructionRefusals.events * 10 + 3;
        cf11.ConditionalConstructionRefusals.effectCalls = cf11.ConditionalConstructionRefusals.effectCalls + 1;
        return;
    }

    public static java.lang.String localBoundary(boolean selected, java.lang.String freshValue, java.lang.String original) {
        // jarde: not recovered: the recovery run for `localBoundary(ZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method localBoundary(ZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `cf11.ConditionalConstructionRefusals`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 8 23 5 2
        // the copy at BCI 11 has no proved local assignment
        // @bytecode 24
        // the value at BCI 24 is the entry state of stack depth 0, which no instruction produced
        // @bytecode 25 26
        // the statement at BCI 26 reads `local`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
    }

    public static java.lang.String sharedFieldAndReturn(boolean selected, java.lang.String freshValue, java.lang.String original) {
        // jarde: not recovered: the recovery run for `sharedFieldAndReturn(ZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method sharedFieldAndReturn(ZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `cf11.ConditionalConstructionRefusals`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 8 23 5 2
        // the copy at BCI 11 has no proved local assignment
        // @bytecode 24
        // the instruction at BCI 24 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 25
        // the copy at BCI 24 has no proved local assignment
        // @bytecode 28
        // the copy at BCI 24 has no proved local assignment
    }

    public static java.lang.String nestedConditionals(boolean outerSelected, boolean innerSelected, java.lang.String innerFresh, java.lang.String innerOriginal, java.lang.String outerFresh) {
        // jarde: not recovered: the recovery run for `nestedConditionals(ZZLjava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method nestedConditionals(ZZLjava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `cf11.ConditionalConstructionRefusals`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 8 16 31 35 5 2
        // the copy at BCI 19 has no proved local assignment
        // @bytecode 48
        // the value at BCI 48 is the entry state of stack depth 0, which no instruction produced
    }

    public static java.lang.String independentBranchEffect(boolean selected, java.lang.String freshValue, java.lang.String original) {
        // @method independentBranchEffect(ZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `cf11.ConditionalConstructionRefusals`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String result;
        if (condition(selected, false)) {
            independentEffect();
            result = new java.lang.String((java.lang.String) argument(freshValue, false));
        } else {
            result = original;
        }
        return result;
    }

    public static java.lang.String handlerAroundConditional(boolean selected, boolean argumentFails, java.lang.String freshValue, java.lang.String original) {
        // @method handlerAroundConditional(ZZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `cf11.ConditionalConstructionRefusals`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (condition(selected, false)) {
                // @bytecode 8
                // the instruction at BCI 8 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
                // @bytecode 11
                // the instruction at BCI 11 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
                // @bytecode 12 13 14
                // the saved producer at BCI 14 has no bounded final expression consumer
                // @bytecode 12 13 14 17
                // the saved producer at BCI 17 has no bounded final expression consumer
            }
        } catch (java.lang.IllegalArgumentException error) {
            cf11.ConditionalConstructionRefusals.events = cf11.ConditionalConstructionRefusals.events * 10 + 4;
            return original;
        }
        // @bytecode 24
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [24]
    }
}
