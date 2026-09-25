// jarde: presentation of `demo/Plain` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package demo;

public enum Plain {
    public static final demo.Plain READY;

    public static final demo.Plain WAITING;

    private static final demo.Plain[] $VALUES;

    public static demo.Plain[] values() {
        // @method values()[Ldemo/Plain;
        // @declaration a static method of `demo.Plain`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (demo.Plain[]) demo.Plain.$VALUES.clone();
    }

    public static demo.Plain valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)Ldemo/Plain;
        // @declaration a static method of `demo.Plain`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (demo.Plain) java.lang.Enum.valueOf(demo.Plain.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private Plain(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `demo.Plain`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        return;
    }

    public java.lang.String label() {
        // @method label()Ljava/lang/String;
        // @declaration an instance method of `demo.Plain`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.name().toLowerCase();
    }

    private static demo.Plain[] $values() {
        // @method $values()[Ldemo/Plain;
        // @declaration a static method of `demo.Plain`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new demo.Plain[]{demo.Plain.READY, demo.Plain.WAITING};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `demo.Plain`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        demo.Plain.READY = new demo.Plain("READY", 0);
        demo.Plain.WAITING = new demo.Plain("WAITING", 1);
        demo.Plain.$VALUES = $values();
    }
}
