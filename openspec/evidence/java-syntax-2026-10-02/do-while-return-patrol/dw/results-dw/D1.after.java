// jarde: presentation of `D1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class D1 extends java.lang.Object {
    static int hits;

    public D1() {
        // @method <init>()V
        // @declaration a constructor of `D1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String retInDoWhile() {
        // @method retInDoWhile()Ljava/lang/String;
        // @declaration a static method of `D1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local0;
        local0 = 0;
        while (local0 < 10) {
            local0 = local0 + 1;
            if (local0 % 3 == 0) {
            } else if (local0 > 7) {
                return new java.lang.StringBuilder().append("early:").append(D1.hits).toString();
    } else {
                D1.hits = D1.hits + 1;
    }
        }
        return "i=" + local0;
    }

    public static java.lang.String retInPlainDo() {
        // @method retInPlainDo()Ljava/lang/String;
        // @declaration a static method of `D1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local0;
        local0 = 0;
        while (local0 != 3) {
            local0 = local0 + 1;
            if (local0 >= 5) {
                return "d" + local0;
            }
        }
        return "d3";
    }

    public static java.lang.String retInIf() {
        // @method retInIf()Ljava/lang/String;
        // @declaration a static method of `D1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local0;
        for (local0 = 0; local0 < 5; local0 = local0 + 1) {
            if (local0 == 2) {
                return "f2";
            }
        }
        return "end";
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `D1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) retInDoWhile());
        java.lang.System.out.println((java.lang.String) retInPlainDo());
        java.lang.System.out.println((java.lang.String) retInIf());
        return;
    }
}
