// jarde: presentation of `V3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class V3 extends java.lang.Object {
    public V3() {
        // @method <init>()V
        // @declaration a constructor of `V3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Object join(boolean arg0) {
        // @method join(Z)Ljava/lang/Object;
        // @declaration a static method of `V3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] local1;
        if (arg0) {
            local1 = new int[]{1, 2, 3};
        } else {
            local1 = new boolean[]{true};
        }
        return local1;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `V3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) join(true).getClass().getName());
        return;
    }
}
