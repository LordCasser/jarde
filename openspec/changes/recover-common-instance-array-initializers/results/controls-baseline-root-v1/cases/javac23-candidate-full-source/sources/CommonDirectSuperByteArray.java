// jarde: presentation of `CommonDirectSuperByteArray` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class CommonDirectSuperByteArray extends java.lang.Object {
    static java.lang.String trace = "";

    byte[] bytes;

    static byte mark(int value) {
        // @method mark(I)B
        // @declaration a static method of `CommonDirectSuperByteArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        CommonDirectSuperByteArray.trace = new java.lang.StringBuilder().append(CommonDirectSuperByteArray.trace).append("eval:").append(value).append(";").toString();
        return (byte) value;
    }

    public CommonDirectSuperByteArray() {
        // @method <init>()V
        // @declaration a constructor of `CommonDirectSuperByteArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.bytes = new byte[]{mark(10), mark(20)};
        CommonDirectSuperByteArray.trace = new java.lang.StringBuilder().append(CommonDirectSuperByteArray.trace).append("body:noarg;").toString();
        return;
    }

    public CommonDirectSuperByteArray(int marker) {
        // @method <init>(I)V
        // @declaration a constructor of `CommonDirectSuperByteArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.bytes = new byte[]{mark(10), mark(20)};
        CommonDirectSuperByteArray.trace = new java.lang.StringBuilder().append(CommonDirectSuperByteArray.trace).append("body:int:").append(marker).append(";").toString();
        return;
    }
}
