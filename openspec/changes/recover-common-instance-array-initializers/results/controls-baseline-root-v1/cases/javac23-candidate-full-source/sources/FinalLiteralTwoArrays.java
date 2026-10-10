// jarde: presentation of `FinalLiteralTwoArrays` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class FinalLiteralTwoArrays extends java.lang.Object {
    static java.lang.String trace = "";

    final byte[] first;

    final byte[] second;

    public FinalLiteralTwoArrays() {
        // @method <init>()V
        // @declaration a constructor of `FinalLiteralTwoArrays`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.first = new byte[]{1, 2};
        this.second = new byte[]{3, 4};
        FinalLiteralTwoArrays.trace = new java.lang.StringBuilder().append(FinalLiteralTwoArrays.trace).append("body:noarg;").toString();
        return;
    }

    public FinalLiteralTwoArrays(int marker) {
        // @method <init>(I)V
        // @declaration a constructor of `FinalLiteralTwoArrays`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.first = new byte[]{1, 2};
        this.second = new byte[]{3, 4};
        FinalLiteralTwoArrays.trace = new java.lang.StringBuilder().append(FinalLiteralTwoArrays.trace).append("body:int:").append(marker).append(";").toString();
        return;
    }
}
