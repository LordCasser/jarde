// jarde: presentation of `W2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class W2 extends java.lang.Object {
    public W2() {
        // @method <init>()V
        // @declaration a constructor of `W2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int noCont(int arg0) {
        // @method noCont(I)I
        // @declaration a static method of `W2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        local2 = 0;
        while (local2 < arg0) {
            switch (local2 % 3) {
                case 0:
                    local1 = local1 + 1;
                    break;
                case 1:
                    local1 = local1 + 2;
                    break;
                default:
                    local1 = local1 + 3;
                    break;
            }
            local1 = local1 + 10;
            local2 = local2 + 1;
        }
        return local1;
    }

    public static int contNoJoin(int arg0) {
        // jarde: not recovered: the recovery run for `contNoJoin(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method contNoJoin(I)I
        // @declaration a static method of `W2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 4 5 6 9 10 11 12 40 43 46 49 52 53 54 57 60 63 66 69 70
        // canonical block at BCI 9 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `W2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(noCont(6));
        java.lang.System.out.println(contNoJoin(6));
        return;
    }
}
