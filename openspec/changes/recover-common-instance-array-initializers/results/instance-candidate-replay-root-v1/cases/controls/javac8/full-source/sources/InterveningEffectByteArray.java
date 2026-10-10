// jarde: presentation of `InterveningEffectByteArray` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class InterveningEffectByteArray extends java.lang.Object {
    static java.lang.String trace = "";

    byte[] bytes;

    static byte mark(int value) {
        // @method mark(I)B
        // @declaration a static method of `InterveningEffectByteArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        InterveningEffectByteArray.trace = new java.lang.StringBuilder().append(InterveningEffectByteArray.trace).append("eval:").append(value).append(";").toString();
        return (byte) value;
    }

    public InterveningEffectByteArray() {
        // @method <init>()V
        // @declaration a constructor of `InterveningEffectByteArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        InterveningEffectByteArray.trace = new java.lang.StringBuilder().append(InterveningEffectByteArray.trace).append("gap;").toString();
        this.bytes = new byte[]{mark(61)};
        InterveningEffectByteArray.trace = new java.lang.StringBuilder().append(InterveningEffectByteArray.trace).append("body:noarg;").toString();
        return;
    }

    public InterveningEffectByteArray(int marker) {
        // @method <init>(I)V
        // @declaration a constructor of `InterveningEffectByteArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.bytes = new byte[]{mark(61)};
        InterveningEffectByteArray.trace = new java.lang.StringBuilder().append(InterveningEffectByteArray.trace).append("body:int:").append(marker).append(";").toString();
        return;
    }
}
