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
        // @method oneConstructedArm(ZZZLjava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `cf11.ConditionalConstructionPositive`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return condition(selected, conditionFails) ? new java.lang.String((java.lang.String) argument(original, argumentFails)) : original;
    }

    public static java.lang.String twoConstructedArms(boolean firstSelected, boolean conditionFails, boolean firstArgumentFails, boolean secondArgumentFails, java.lang.String firstValue, java.lang.String secondValue) {
        // @method twoConstructedArms(ZZZZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `cf11.ConditionalConstructionPositive`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return condition(firstSelected, conditionFails) ? new java.lang.String((java.lang.String) argument(firstValue, firstArgumentFails)) : new java.lang.String((java.lang.String) argument(secondValue, secondArgumentFails));
    }

    public static void writeStaticSlot(boolean selected, boolean conditionFails, boolean argumentFails, java.lang.String original) {
        // @method writeStaticSlot(ZZZLjava/lang/String;)V
        // @declaration a static method of `cf11.ConditionalConstructionPositive`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        cf11.ConditionalConstructionPositive.staticSlot = condition(selected, conditionFails) ? new java.lang.String((java.lang.String) argument(original, argumentFails)) : original;
        return;
    }

    public void writeInstanceSlot(boolean selected, boolean conditionFails, boolean argumentFails, java.lang.String original) {
        // @method writeInstanceSlot(ZZZLjava/lang/String;)V
        // @declaration an instance method of `cf11.ConditionalConstructionPositive`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.instanceSlot = condition(selected, conditionFails) ? new java.lang.String((java.lang.String) argument(original, argumentFails)) : original;
        return;
    }
}
