// jarde: presentation of `V4` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class V4 extends java.lang.Object {
    public V4() {
        // @method <init>()V
        // @declaration a constructor of `V4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Object loop(int arg0) {
        // @method loop(I)Ljava/lang/Object;
        // @declaration a static method of `V4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] local1;
        int local2;
        local1 = new int[]{1, 2, 3};
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            local1 = new boolean[]{true};
        }
        return local1;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `V4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) loop(0).getClass().getName());
        return;
    }
}
