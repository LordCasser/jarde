// jarde: presentation of `cf11/ConditionalConstructionPositive` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class ConditionalConstructionPositive extends java.lang.Object {
    public static int events;

    public static int conditionCalls;

    public static int argumentCalls;

    public static java.lang.String staticSlot;

    public java.lang.String instanceSlot;

    public ConditionalConstructionPositive() {
        // @method <init>()V
        // @declaration a constructor of `cf11.ConditionalConstructionPositive`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void reset() {
        // @method reset()V
        // @declaration a static method of `cf11.ConditionalConstructionPositive`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        cf11.ConditionalConstructionPositive.events = 0;
        cf11.ConditionalConstructionPositive.conditionCalls = 0;
        cf11.ConditionalConstructionPositive.argumentCalls = 0;
        cf11.ConditionalConstructionPositive.staticSlot = null;
        return;
    }

    private static boolean condition(boolean selected, boolean fail) {
        // @method condition(ZZ)Z
        // @declaration a static method of `cf11.ConditionalConstructionPositive`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        cf11.ConditionalConstructionPositive.events = cf11.ConditionalConstructionPositive.events * 10 + 1;
        cf11.ConditionalConstructionPositive.conditionCalls = cf11.ConditionalConstructionPositive.conditionCalls + 1;
        if (fail) {
            throw new java.lang.IllegalStateException("condition");
        } else {
            return selected;
        }
    }

    private static java.lang.String argument(java.lang.String value, boolean fail) {
        // @method argument(Ljava/lang/String;Z)Ljava/lang/String;
        // @declaration a static method of `cf11.ConditionalConstructionPositive`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        cf11.ConditionalConstructionPositive.events = cf11.ConditionalConstructionPositive.events * 10 + 2;
        cf11.ConditionalConstructionPositive.argumentCalls = cf11.ConditionalConstructionPositive.argumentCalls + 1;
        if (fail) {
            throw new java.lang.IllegalArgumentException("argument");
        } else {
            return value;
        }
    }

    public static java.lang.String oneConstructedArm(boolean selected, boolean conditionFails, boolean argumentFails, java.lang.String original) {
        // jarde: not recovered: the recovery run for `oneConstructedArm(ZZZLjava/lang/String;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method oneConstructedArm(ZZZLjava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `cf11.ConditionalConstructionPositive`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 8 23 5 2 1
        // the copy at BCI 11 has no proved local assignment
        // @bytecode 24
        // the value at BCI 24 is the entry state of stack depth 0, which no instruction produced
    }

    public static java.lang.String twoConstructedArms(boolean firstSelected, boolean conditionFails, boolean firstArgumentFails, boolean secondArgumentFails, java.lang.String firstValue, java.lang.String secondValue) {
        // jarde: not recovered: the recovery run for `twoConstructedArms(ZZZZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method twoConstructedArms(ZZZZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `cf11.ConditionalConstructionPositive`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 8 24 5 2 1
        // the copy at BCI 11 has no proved local assignment
        // @bytecode 37
        // the value at BCI 37 is the entry state of stack depth 0, which no instruction produced
    }

    public static void writeStaticSlot(boolean selected, boolean conditionFails, boolean argumentFails, java.lang.String original) {
        // jarde: not recovered: the recovery run for `writeStaticSlot(ZZZLjava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method writeStaticSlot(ZZZLjava/lang/String;)V
        // @declaration a static method of `cf11.ConditionalConstructionPositive`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 8 23 5 2 1 11 12 13 14 17 20 24 27
        // the copy at BCI 11 has no proved local assignment
        // @bytecode 24
        // the value at BCI 24 is the entry state of stack depth 0, which no instruction produced
        jarde_refused_body();
    }

    public void writeInstanceSlot(boolean selected, boolean conditionFails, boolean argumentFails, java.lang.String original) {
        // jarde: not recovered: the recovery run for `writeInstanceSlot(ZZZLjava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method writeInstanceSlot(ZZZLjava/lang/String;)V
        // @declaration an instance method of `cf11.ConditionalConstructionPositive`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 9 25 6 3 1 2 12 13 15 16 19 22 27 30
        // the copy at BCI 12 has no proved local assignment
        // @bytecode 27
        // the value at BCI 27 is the entry state of stack depth 1, which no instruction produced
        jarde_refused_body();
    }
}
