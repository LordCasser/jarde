// jarde: presentation of `cf11/ForConditionReads` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class ForConditionReads extends java.lang.Object {
    static int fieldBound;

    static int negativeField;

    public ForConditionReads() {
        // @method <init>()V
        // @declaration a constructor of `cf11.ForConditionReads`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int sumWithChangingFieldBound() {
        // @method sumWithChangingFieldBound()I
        // @declaration a static method of `cf11.ForConditionReads`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        int i;
        sum = 0;
        for (i = 0; i < cf11.ForConditionReads.fieldBound; i = i + 1) {
            sum = sum + i;
            cf11.ForConditionReads.fieldBound = cf11.ForConditionReads.fieldBound - 1;
        }
        return sum;
    }

    static int sumWithArrayLength(int[] values) {
        // @method sumWithArrayLength([I)I
        // @declaration a static method of `cf11.ForConditionReads`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        int i;
        sum = 0;
        for (i = 0; i < values.length; i = i + 1) {
            sum = sum + i;
        }
        return sum;
    }

    static int countWithArrayElement(int[] values) {
        // @method countWithArrayElement([I)I
        // @declaration a static method of `cf11.ForConditionReads`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int count;
        int i;
        count = 0;
        for (i = 0; i < values[i]; i = i + 1) {
            count = count + 1;
        }
        return count;
    }

    static int sumWithNegatedField() {
        // @method sumWithNegatedField()I
        // @declaration a static method of `cf11.ForConditionReads`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        int i;
        sum = 0;
        for (i = 0; i < -cf11.ForConditionReads.negativeField; i = i + 1) {
            sum = sum + i;
        }
        return sum;
    }

    static int sumWithNegatedParameter(int value) {
        // @method sumWithNegatedParameter(I)I
        // @declaration a static method of `cf11.ForConditionReads`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        int i;
        sum = 0;
        for (i = 0; i < -value; i = i + 1) {
            sum = sum + i;
        }
        return sum;
    }

    static int oldWhileWithWrittenLocalBound() {
        // @method oldWhileWithWrittenLocalBound()I
        // @declaration a static method of `cf11.ForConditionReads`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int bound;
        int i;
        int count;
        bound = 3;
        i = 0;
        count = 0;
        while (i < bound) {
            count = count + 1;
            bound = bound - 1;
            i = i + 1;
        }
        return count;
    }
}
