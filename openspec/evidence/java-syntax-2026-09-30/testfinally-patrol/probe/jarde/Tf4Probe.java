// jarde: presentation of `Tf4Probe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Tf4Probe extends java.lang.Object {
    public static boolean failCall;

    public int result;

    public Tf4Probe() {
        // @method <init>()V
        // @declaration a constructor of `Tf4Probe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.result = 0;
        return;
    }

    public java.lang.String test() {
        // @method test()Ljava/lang/String;
        // @declaration an instance method of `Tf4Probe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        boolean local1;
        java.lang.String local3;
        local1 = false;
        try {
            java.lang.String local2 = this.call();
            this.result += 1;
            local1 = true;
            local3 = local2;
            return local3;
        } finally {
            if (!local1) {
                this.result -= 2;
            }
        }
    }

    private java.lang.String call() {
        // @method call()Ljava/lang/String;
        // @declaration an instance method of `Tf4Probe`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        if (Tf4Probe.failCall) {
            throw new java.lang.RuntimeException("call");
        } else {
            return "call";
        }
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `Tf4Probe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        Tf4Probe.failCall = false;
    }
}
