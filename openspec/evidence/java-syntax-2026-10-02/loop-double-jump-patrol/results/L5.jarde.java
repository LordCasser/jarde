// jarde: presentation of `L5` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class L5 extends java.lang.Object {
    public L5() {
        // @method <init>()V
        // @declaration a constructor of `L5`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String brkSelfContSelf(int arg0) {
        // jarde: not recovered: the recovery run for `brkSelfContSelf(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method brkSelfContSelf(I)Ljava/lang/String;
        // @declaration a static method of `L5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 10 15 17 22 25 31 37 40 45 48 68 74 80 86
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String brkSelfContMid(int arg0) {
        // jarde: not recovered: the recovery run for `brkSelfContMid(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method brkSelfContMid(I)Ljava/lang/String;
        // @declaration a static method of `L5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 10 15 17 22 25 31 37 40 45 48 68 74 80 86
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String dblJumpDoWhile() {
        // jarde: not recovered: the recovery run for `dblJumpDoWhile()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method dblJumpDoWhile()Ljava/lang/String;
        // @declaration a static method of `L5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 5 8 11 12 13 14 17 20 21 23 26 29 32 33 36 38 41 42 45 48
        // canonical block at BCI 2 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }

    public static java.lang.String dblJumpDoWhilePlain() {
        // @method dblJumpDoWhilePlain()Ljava/lang/String;
        // @declaration a static method of `L5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local0;
        local0 = 0;
        while (local0 < 10) {
            local0 = local0 + 1;
            if (local0 % 3 == 0) {
            } else if (local0 > 7) {
    }
        }
        return "i=" + local0;
        // @bytecode 26 28
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [26]
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `L5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) brkSelfContSelf(2));
        java.lang.System.out.println((java.lang.String) brkSelfContMid(3));
        java.lang.System.out.println((java.lang.String) dblJumpDoWhile());
        java.lang.System.out.println((java.lang.String) dblJumpDoWhilePlain());
        return;
    }
}
