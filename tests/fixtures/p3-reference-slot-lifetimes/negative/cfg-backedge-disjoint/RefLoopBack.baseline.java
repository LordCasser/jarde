// jarde: presentation of `RefLoopBack` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class RefLoopBack extends java.lang.Object {
    public RefLoopBack() {
        // @method <init>()V
        // @declaration a constructor of `RefLoopBack`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int run(int arg0) {
        // @method run(I)I
        // @declaration a static method of `RefLoopBack`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = 0;
        while (arg0-- > 0) {
            java.lang.String local2 = "first";
            local1 = local1 + local2.length();
            local2 = new java.lang.StringBuilder("second");
            local1 = local1 + local2.length();
        }
        return local1;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `RefLoopBack`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(run(2));
        return;
    }
}
