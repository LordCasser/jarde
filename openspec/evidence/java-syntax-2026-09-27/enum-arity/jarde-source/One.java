// jarde: presentation of `probe/One` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package probe;

public enum One {
    public static final probe.One ONLY;

    private static final probe.One[] $VALUES;

    public static probe.One[] values() {
        // @method values()[Lprobe/One;
        // @declaration a static method of `probe.One`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (probe.One[]) probe.One.$VALUES.clone();
    }

    public static probe.One valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)Lprobe/One;
        // @declaration a static method of `probe.One`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (probe.One) java.lang.Enum.valueOf(probe.One.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private One(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `probe.One`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        return;
    }

    private static probe.One[] $values() {
        // @method $values()[Lprobe/One;
        // @declaration a static method of `probe.One`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new probe.One[]{probe.One.ONLY};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `probe.One`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        probe.One.ONLY = new probe.One("ONLY", 0);
        probe.One.$VALUES = $values();
    }
}
