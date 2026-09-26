// jarde: presentation of `probe/Four` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package probe;

public enum Four {
    public static final probe.Four NORTH;

    public static final probe.Four SOUTH;

    public static final probe.Four EAST;

    public static final probe.Four WEST;

    private static final probe.Four[] $VALUES;

    public static probe.Four[] values() {
        // @method values()[Lprobe/Four;
        // @declaration a static method of `probe.Four`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (probe.Four[]) probe.Four.$VALUES.clone();
    }

    public static probe.Four valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)Lprobe/Four;
        // @declaration a static method of `probe.Four`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (probe.Four) java.lang.Enum.valueOf(probe.Four.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private Four(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `probe.Four`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        return;
    }

    private static probe.Four[] $values() {
        // @method $values()[Lprobe/Four;
        // @declaration a static method of `probe.Four`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new probe.Four[]{probe.Four.NORTH, probe.Four.SOUTH, probe.Four.EAST, probe.Four.WEST};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `probe.Four`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        probe.Four.NORTH = new probe.Four("NORTH", 0);
        probe.Four.SOUTH = new probe.Four("SOUTH", 1);
        probe.Four.EAST = new probe.Four("EAST", 2);
        probe.Four.WEST = new probe.Four("WEST", 3);
        probe.Four.$VALUES = $values();
    }
}
