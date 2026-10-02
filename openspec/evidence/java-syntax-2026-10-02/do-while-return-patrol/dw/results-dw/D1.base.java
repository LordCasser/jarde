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
    } else {
                D1.hits = D1.hits + 1;
    }
        }
        return "i=" + local0;
        // @bytecode 26 29 30 33 35 38 41 44 47
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [26]
    }

    public static java.lang.String retInPlainDo() {
        // jarde: not recovered: the recovery run for `retInPlainDo()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method retInPlainDo()Ljava/lang/String;
        // @declaration a static method of `D1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 2 7 10 18
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String retInIf() {
        // @method retInIf()Ljava/lang/String;
        // @declaration a static method of `D1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local0;
        for (local0 = 0; local0 < 5; local0 = local0 + 1) {
            if (local0 == 2) {
            }
        }
        return "end";
        // @bytecode 12 14
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [12]
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
