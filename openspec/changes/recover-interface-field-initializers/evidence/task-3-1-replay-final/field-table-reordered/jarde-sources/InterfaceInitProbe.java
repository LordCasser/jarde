// jarde: presentation of `InterfaceInitProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public interface InterfaceInitProbe {
    public static final java.lang.String FIRST = InitEffects.next("A");

    public static final java.lang.String SECOND = InitEffects.next("B");

    public static final int TOTAL = InitEffects.total(InterfaceInitProbe.FIRST, InterfaceInitProbe.SECOND);

    public static final int CONSTANT = 7;

    public static java.lang.String observe() {
        // @method observe()Ljava/lang/String;
        // @declaration an interface's static method of `InterfaceInitProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append(InitEffects.trace).append("|").append(InterfaceInitProbe.FIRST).append("|").append(InterfaceInitProbe.SECOND).append("|").append(InterfaceInitProbe.TOTAL).append("|").append(7).toString();
    }
}
