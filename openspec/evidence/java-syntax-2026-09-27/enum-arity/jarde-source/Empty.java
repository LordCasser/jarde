// jarde: presentation of `probe/Empty` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package probe;

public enum Empty {
    private static final probe.Empty[] $VALUES;

    public static probe.Empty[] values() {
        // @method values()[Lprobe/Empty;
        // @declaration a static method of `probe.Empty`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (probe.Empty[]) probe.Empty.$VALUES.clone();
    }

    public static probe.Empty valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)Lprobe/Empty;
        // @declaration a static method of `probe.Empty`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (probe.Empty) java.lang.Enum.valueOf(probe.Empty.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private Empty(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `probe.Empty`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        return;
    }

    private static probe.Empty[] $values() {
        // @method $values()[Lprobe/Empty;
        // @declaration a static method of `probe.Empty`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new probe.Empty[0];
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `probe.Empty`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        probe.Empty.$VALUES = $values();
    }
}
