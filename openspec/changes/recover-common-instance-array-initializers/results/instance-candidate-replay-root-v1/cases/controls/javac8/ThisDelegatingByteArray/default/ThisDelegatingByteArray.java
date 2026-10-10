// jarde: presentation of `ThisDelegatingByteArray` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ThisDelegatingByteArray extends java.lang.Object {
    static java.lang.String trace = "";

    byte[] bytes;

    static byte mark(int value) {
        // @method mark(I)B
        // @declaration a static method of `ThisDelegatingByteArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ThisDelegatingByteArray.trace = new java.lang.StringBuilder().append(ThisDelegatingByteArray.trace).append("eval:").append(value).append(";").toString();
        return (byte) value;
    }

    public ThisDelegatingByteArray() {
        // @method <init>()V
        // @declaration a constructor of `ThisDelegatingByteArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this(7);
        ThisDelegatingByteArray.trace = new java.lang.StringBuilder().append(ThisDelegatingByteArray.trace).append("body:delegate;").toString();
        return;
    }

    public ThisDelegatingByteArray(int marker) {
        // @method <init>(I)V
        // @declaration a constructor of `ThisDelegatingByteArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.bytes = new byte[]{mark(21), mark(22)};
        ThisDelegatingByteArray.trace = new java.lang.StringBuilder().append(ThisDelegatingByteArray.trace).append("body:target:").append(marker).append(";").toString();
        return;
    }
}
