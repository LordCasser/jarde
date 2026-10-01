// jarde: presentation of `B4` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class B4 extends java.lang.Object {
    public B4() {
        // @method <init>()V
        // @declaration a constructor of `B4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String v1(int arg0) {
        // @method v1(I)Ljava/lang/String;
        // @declaration a static method of `B4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1 = arg0;
        local1 = local1 | 16;
        local1 = local1 & 60;
        local1 = local1 ^ 8;
        local1 = local1 << 1;
        local1 = local1 >> 2;
        boolean local2 = (arg0 & 1) != 0 && (arg0 & 2) != 0;
        return "" + local1 + ":" + local2;
    }

    public static java.lang.String v2(int arg0) {
        // @method v2(I)Ljava/lang/String;
        // @declaration a static method of `B4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1 = arg0;
        local1 = local1 | 16;
        boolean local2 = (arg0 & 1) != 0 && (arg0 & 2) != 0;
        boolean local3 = (arg0 & 4) != 0 || (arg0 & 8) != 0;
        return "" + local1 + ":" + local2 + ":" + local3;
    }

    public static java.lang.String v3(int arg0) {
        // @method v3(I)Ljava/lang/String;
        // @declaration a static method of `B4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local1 = (arg0 & 1) != 0 && (arg0 & 2) != 0;
        boolean local2 = (arg0 & 4) != 0 || (arg0 & 8) != 0;
        return "" + local1 + ":" + local2;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `B4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) v1(27));
        java.lang.System.out.println((java.lang.String) v2(27));
        java.lang.System.out.println((java.lang.String) v3(27));
        return;
    }
}
