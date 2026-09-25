// jarde: presentation of `ExceptionProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public interface ExceptionProbe {
    public static final java.lang.String FIRST;

    public static final java.lang.String SECOND;

    public static java.lang.String observe() {
        // @method observe()Ljava/lang/String;
        // @declaration an interface's static method of `ExceptionProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append(BoundaryEffects.trace).append("|").append(ExceptionProbe.FIRST).append("|").append(ExceptionProbe.SECOND).toString();
    }

    static {
        // jarde: not recovered: the recovery run for `<clinit>()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method <clinit>()V
        // @declaration a static initializer of `ExceptionProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // BCI 9: the resource's own initialisation is not one statement of this block whose value lands in a slot: writing it in the header would move or drop an effect
        // @bytecode 22 18
        // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [22, 18]
    }
}
