// jarde: presentation of `cf11/ForConditionNumeric` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class ForConditionNumeric extends java.lang.Object {
    public ForConditionNumeric() {
        // @method <init>()V
        // @declaration a constructor of `cf11.ForConditionNumeric`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int countLong(long[] values, long bound) {
        // @method countLong([JJ)I
        // @declaration a static method of `cf11.ForConditionNumeric`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int count;
        int i;
        count = 0;
        for (i = 0; values[i] < bound; i = i + 1) {
            count = count + 1;
        }
        return count;
    }

    static int countFloat(float[] values, float bound) {
        // @method countFloat([FF)I
        // @declaration a static method of `cf11.ForConditionNumeric`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int count;
        int i;
        count = 0;
        for (i = 0; values[i] < bound; i = i + 1) {
            count = count + 1;
        }
        return count;
    }

    static int countDouble(double[] values, double bound) {
        // @method countDouble([DD)I
        // @declaration a static method of `cf11.ForConditionNumeric`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int count;
        int i;
        count = 0;
        for (i = 0; values[i] < bound; i = i + 1) {
            count = count + 1;
        }
        return count;
    }
}
