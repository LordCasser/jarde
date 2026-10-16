// jarde: presentation of `FO` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class FO extends java.lang.Object {
    static int i = 0;

    public FO() {
        // @method <init>()V
        // @declaration a constructor of `FO`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int bump() {
        // @method bump()I
        // @declaration a static method of `FO`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return ++FO.i;
    }

    static int evalThenFin() {
        // @method evalThenFin()I
        // @declaration a static method of `FO`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            int local0 = bump();
            return local0;
        } finally {
            FO.i = 100;
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `FO`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(evalThenFin()).append("/").append(FO.i).toString());
        return;
    }
}
