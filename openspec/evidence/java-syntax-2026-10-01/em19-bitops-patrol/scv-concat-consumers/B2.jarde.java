// jarde: presentation of `B2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class B2 extends java.lang.Object {
    static long seed;

    public B2() {
        // @method <init>()V
        // @declaration a constructor of `B2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static long longOps(long arg0) {
        // @method longOps(J)J
        // @declaration a static method of `B2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        long local2 = arg0;
        local2 = local2 ^ 65535L;
        local2 = local2 | (arg0 & 255L) << 8;
        local2 = local2 & -16L;
        local2 = local2 >>> 2;
        return local2;
    }

    public static java.lang.String compound(int arg0) {
        // @method compound(I)Ljava/lang/String;
        // @declaration a static method of `B2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1 = arg0;
        local1 = local1 | 16;
        local1 = local1 & 60;
        local1 = local1 ^ 8;
        local1 = local1 << 1;
        local1 = local1 >> 2;
        boolean local2 = (arg0 & 1) != 0 && (arg0 & 2) != 0;
        boolean local3 = (arg0 & 4) != 0 || (arg0 & 8) != 0;
        return "" + local1 + ":" + local2 + ":" + local3;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `B2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(longOps(B2.seed));
        java.lang.System.out.println((java.lang.String) compound(27));
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `B2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        B2.seed = 78187493530L;
    }
}
