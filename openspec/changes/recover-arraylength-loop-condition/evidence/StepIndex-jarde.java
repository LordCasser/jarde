// jarde: presentation of `StepIndex` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class StepIndex extends java.lang.Object {
    public StepIndex() {
        // @method <init>()V
        // @declaration a constructor of `StepIndex`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int everyOther(int[] values) {
        // @method everyOther([I)I
        // @declaration a static method of `StepIndex`, member flags 0x0009
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
        // @declaration a static method of `StepIndex`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.io.PrintStream saved0 = java.lang.System.out;
        saved0.println(everyOther(new int[]{1, 2, 3, 4}));
        return;
    }
}
