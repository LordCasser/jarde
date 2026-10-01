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
        // jarde: not recovered: the recovery run for `compound(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method compound(I)Ljava/lang/String;
        // @declaration a static method of `B2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 5 6 7 8 10 11 12 13 15 16 17 18 19 20 21 22 23 24 25 26 27 28 31 32 33 34 37 38 41 42 43 44 45 46 49 50 52 53 56 57 60 61 62 65 66 69 70 73 75 78 79 82 84 87 88 91 94
        // the arms of the branch in block 31 do not meet at one join
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
