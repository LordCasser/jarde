// jarde: presentation of `cf11/ForConditionEffects` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class ForConditionEffects extends java.lang.Object {
    static int events;

    static int offsetCalls;

    static int limitCalls;

    static int offsetValue;

    static int limitValue;

    static int offsetThrowAt;

    static int limitThrowAt;

    static int dynamicLimit;

    public ForConditionEffects() {
        // @method <init>()V
        // @declaration a constructor of `cf11.ForConditionEffects`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int offset() {
        // @method offset()I
        // @declaration a static method of `cf11.ForConditionEffects`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        cf11.ForConditionEffects.offsetCalls = cf11.ForConditionEffects.offsetCalls + 1;
        cf11.ForConditionEffects.events = cf11.ForConditionEffects.events * 10 + 1;
        if (cf11.ForConditionEffects.offsetThrowAt != 0) {
            if (cf11.ForConditionEffects.offsetCalls == cf11.ForConditionEffects.offsetThrowAt) {
                throw new java.lang.IllegalStateException("offset");
            }
        }
        return cf11.ForConditionEffects.offsetValue;
    }

    static int limit() {
        // @method limit()I
        // @declaration a static method of `cf11.ForConditionEffects`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        cf11.ForConditionEffects.limitCalls = cf11.ForConditionEffects.limitCalls + 1;
        cf11.ForConditionEffects.events = cf11.ForConditionEffects.events * 10 + 2;
        if (cf11.ForConditionEffects.limitThrowAt != 0) {
            if (cf11.ForConditionEffects.limitCalls == cf11.ForConditionEffects.limitThrowAt) {
                throw new java.lang.IllegalStateException("limit");
            }
        }
        return cf11.ForConditionEffects.limitValue;
    }

    static int sumWithFunctionBounds() {
        // @method sumWithFunctionBounds()I
        // @declaration a static method of `cf11.ForConditionEffects`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        int i;
        sum = 0;
        i = 0;
        while (i < offset() + limit()) {
            sum = sum + i;
            cf11.ForConditionEffects.events = cf11.ForConditionEffects.events * 10 + 3;
            i = i + 1;
        }
        return sum;
    }

    static int sumWithFunctionBoundsAndContinue() {
        // @method sumWithFunctionBoundsAndContinue()I
        // @declaration a static method of `cf11.ForConditionEffects`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        int i;
        sum = 0;
        i = 0;
        while (i < offset() + limit()) {
            if (i == 0) {
                cf11.ForConditionEffects.events = cf11.ForConditionEffects.events * 10 + 4;
            } else {
                sum = sum + i;
                cf11.ForConditionEffects.events = cf11.ForConditionEffects.events * 10 + 3;
            }
            i = i + 1;
        }
        return sum;
    }

    static int sumWithChangingFieldBound() {
        // @method sumWithChangingFieldBound()I
        // @declaration a static method of `cf11.ForConditionEffects`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        int i;
        sum = 0;
        i = 0;
        while (i < cf11.ForConditionEffects.dynamicLimit) {
            sum = sum + i;
            cf11.ForConditionEffects.events = cf11.ForConditionEffects.events * 10 + 3;
            cf11.ForConditionEffects.dynamicLimit = cf11.ForConditionEffects.dynamicLimit - 1;
            i = i + 1;
        }
        return sum;
    }
}
