// jarde: presentation of `ArrayThenList` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ArrayThenList extends java.lang.Object {
    public ArrayThenList() {
        // @method <init>()V
        // @declaration a constructor of `ArrayThenList`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int run(int[] input) {
        // @method run([I)I
        // @declaration a static method of `ArrayThenList`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        int[] local3;
        sum = 0;
        int[] values = input;
        local3 = values;
        for (int value : local3) {
            sum = sum + value;
        }
        java.util.ArrayList out = new java.util.ArrayList();
        out.add((java.lang.Object) java.lang.Integer.valueOf(sum));
        out.add((java.lang.Object) java.lang.Integer.valueOf(9));
        return ((java.lang.Integer) out.get(0)).intValue() + out.size();
    }

    public static void main(java.lang.String[] x) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ArrayThenList`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.io.PrintStream saved0 = java.lang.System.out;
        saved0.println(run(new int[]{2, 4}));
        return;
    }
}
