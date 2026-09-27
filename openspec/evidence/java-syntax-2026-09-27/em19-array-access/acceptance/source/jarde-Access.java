// jarde: presentation of `em19/Access` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em19;

public class Access extends java.lang.Object {
    public Access() {
        // @method <init>()V
        // @declaration a constructor of `em19.Access`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int at(int arg0) {
        // @method at(I)I
        // @declaration a static method of `em19.Access`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] local1 = new int[]{1, 2, 3, 5};
        return local1[arg0];
    }

    public static int dimensions(int arg0) {
        // @method dimensions(I)I
        // @declaration a static method of `em19.Access`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[][] local1 = new int[arg0][arg0 + 1];
        return local1.length;
    }

    public static int[] reverseNegate(int[] arg0) {
        // @method reverseNegate([I)[I
        // @declaration a static method of `em19.Access`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] local2;
        int local3;
        int local4;
        int local1 = arg0.length;
        local2 = new int[local1];
        local3 = 0;
        local4 = local1;
        while (local4 != 0) {
            int local5 = arg0[local3];
            local4 = local4 - 1;
            int local6 = -local5;
            local3 = local3 + 1;
            local2[local4] = local6 * 5;
        }
        return local2;
    }
}
