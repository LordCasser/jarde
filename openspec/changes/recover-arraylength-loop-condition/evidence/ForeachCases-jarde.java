// jarde: presentation of `ForeachCases` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ForeachCases extends java.lang.Object {
    public ForeachCases() {
        // @method <init>()V
        // @declaration a constructor of `ForeachCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int sum(int[] values) {
        // @method sum([I)I
        // @declaration a static method of `ForeachCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int total;
        int[] local2;
        total = 0;
        local2 = values;
        for (int value : local2) {
            total = total + value;
        }
        return total;
    }

    // jarde: generic Signature projection refused for `join(Ljava/lang/Iterable;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    public static java.lang.String join(java.lang.Iterable values) {
        // @method join(Ljava/lang/Iterable;)Ljava/lang/String;
        // @declaration a static method of `ForeachCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder result;
        result = new java.lang.StringBuilder();
        for (java.lang.Object iteratorElement25 : values) {
            java.lang.String value = (java.lang.String) iteratorElement25;
            result.append(value);
        }
        return result.toString();
    }

    public static int everyOther(int[] values) {
        // @method everyOther([I)I
        // @declaration a static method of `ForeachCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int total;
        int i;
        total = 0;
        i = 0;
        while (i < values.length) {
            total = total + values[i];
            i = i + 2;
        }
        return total;
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ForeachCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.io.PrintStream saved0 = java.lang.System.out;
        saved0.println(sum(new int[]{1, 2, 3, 4}));
        java.io.PrintStream saved1 = java.lang.System.out;
        // @bytecode 56 28 53 50 32 31 35 36 37 39 40 41 42 44 45 46 47 49
        // the parameter 0 of the invocation at BCI 53 is declared `java.lang.Iterable` presents `java.util.List` but the invocation requires `java.lang.Iterable` and this layer has no safe reference conversion evidence
        java.io.PrintStream saved2 = java.lang.System.out;
        saved2.println(everyOther(new int[]{1, 2, 3, 4}));
        return;
    }
}
