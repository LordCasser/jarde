// jarde: presentation of `HandlerArrayByteArray` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class HandlerArrayByteArray extends java.lang.Object {
    static java.lang.String trace;

    static boolean fail;

    byte[] bytes;

    static byte mark(int value) {
        // @method mark(I)B
        // @declaration a static method of `HandlerArrayByteArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        HandlerArrayByteArray.trace = new java.lang.StringBuilder().append(HandlerArrayByteArray.trace).append("eval:").append(value).append(";").toString();
        if (HandlerArrayByteArray.fail) {
            throw new java.lang.IllegalStateException("requested failure");
        } else {
            return (byte) value;
        }
    }

    public HandlerArrayByteArray() {
        // @method <init>()V
        // @declaration a constructor of `HandlerArrayByteArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        try {
            this.bytes = new byte[]{mark(91)};
        } catch (java.lang.IllegalStateException ex) {
            HandlerArrayByteArray.trace = new java.lang.StringBuilder().append(HandlerArrayByteArray.trace).append("caught;").toString();
        }
        HandlerArrayByteArray.trace = new java.lang.StringBuilder().append(HandlerArrayByteArray.trace).append("body:noarg;").toString();
        return;
    }

    public HandlerArrayByteArray(int marker) {
        // @method <init>(I)V
        // @declaration a constructor of `HandlerArrayByteArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        try {
            this.bytes = new byte[]{mark(91)};
        } catch (java.lang.IllegalStateException ex) {
            HandlerArrayByteArray.trace = new java.lang.StringBuilder().append(HandlerArrayByteArray.trace).append("caught;").toString();
        }
        HandlerArrayByteArray.trace = new java.lang.StringBuilder().append(HandlerArrayByteArray.trace).append("body:int:").append(marker).append(";").toString();
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `HandlerArrayByteArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        HandlerArrayByteArray.trace = "";
    }
}
