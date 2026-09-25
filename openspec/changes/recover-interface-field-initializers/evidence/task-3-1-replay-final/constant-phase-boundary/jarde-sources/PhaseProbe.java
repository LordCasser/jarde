// jarde: presentation of `PhaseProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public interface PhaseProbe {
    public static final int EARLY;

    public static final int LATE;

    public static java.lang.String observe() {
        // @method observe()Ljava/lang/String;
        // @declaration an interface's static method of `PhaseProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append(PhaseProbe.EARLY).append("|").append(PhaseProbe.LATE).toString();
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `PhaseProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        PhaseProbe.EARLY = PhaseProbe.LATE;
        PhaseProbe.LATE = 9;
    }
}
