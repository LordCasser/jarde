// jarde: presentation of `V2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class V2 extends java.lang.Object {
    public V2() {
        // @method <init>()V
        // @declaration a constructor of `V2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int sameType() {
        // @method sameType()I
        // @declaration a static method of `V2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local0 = 0;
        int[] local1 = new int[2];
        local1[0] = 1;
        local0 = local0 + local1[0];
        local1 = new int[3];
        local1[1] = 2;
        local0 = local0 + local1[1];
        return local0;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `V2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(sameType());
        return;
    }
}
