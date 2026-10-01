// jarde: presentation of `B1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class B1 extends java.lang.Object {
    public B1() {
        // @method <init>()V
        // @declaration a constructor of `B1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String bitOps(int arg0) {
        // @method bitOps(I)Ljava/lang/String;
        // @declaration a static method of `B1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        local1 = new java.lang.StringBuilder();
        if ((arg0 & 240) != 0) {
            local1.append('h');
        }
        if ((arg0 | 1) == arg0 + 1) {
            local1.append('o');
        }
        if ((arg0 ^ 15) == 240) {
            local1.append('x');
        }
        int local2 = arg0 << 2 | arg0 >> 3 | arg0 >>> 1;
        local1.append(':').append(local2 & 255);
        local1.append(':').append((arg0 ^ -1) & 255);
        int local3 = local2 & getMask();
        local1.append(':').append(local3);
        return local1.toString();
    }

    static int getMask() {
        // @method getMask()I
        // @declaration a static method of `B1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return 51;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `B1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) bitOps(90));
        java.lang.System.out.println((java.lang.String) bitOps(3));
        return;
    }
}
