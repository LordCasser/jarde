// jarde: presentation of `em18/Arrays` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em18;

public class Arrays extends java.lang.Object {
    public Arrays() {
        // @method <init>()V
        // @declaration a constructor of `em18.Arrays`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String[] strings() {
        // @method strings()[Ljava/lang/String;
        // @declaration a static method of `em18.Arrays`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.String[]{"1", "2", "3"};
    }

    public static int[] ints(int arg0) {
        // @method ints(I)[I
        // @declaration a static method of `em18.Arrays`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new int[]{1, arg0 + 1, 2};
    }

    public static int[] postfix(int arg0) {
        // @method postfix(I)[I
        // @declaration a static method of `em18.Arrays`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new int[]{1, arg0++, arg0 * 2};
    }

    public static int[] selfRead() {
        // @method selfRead()[I
        // @declaration a static method of `em18.Arrays`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] local0 = new int[3];
        local0[0] = 1;
        local0[1] = local0[0] + 1;
        local0[2] = local0[1] + 1;
        return local0;
    }

    public static int objectArg(java.lang.Exception arg0) {
        // @method objectArg(Ljava/lang/Exception;)I
        // @declaration a static method of `em18.Arrays`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return use(new java.lang.Object[]{arg0});
    }

    private static int use(java.lang.Object[] arg0) {
        // @method use([Ljava/lang/Object;)I
        // @declaration a static method of `em18.Arrays`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.length;
    }
}
