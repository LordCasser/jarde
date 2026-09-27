// jarde: presentation of `NestedStringSwitchAudit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NestedStringSwitchAudit extends java.lang.Object {
    public NestedStringSwitchAudit() {
        // @method <init>()V
        // @declaration a constructor of `NestedStringSwitchAudit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int choose(java.lang.String arg0) {
        // jarde: not recovered: the recovery run for `choose(Ljava/lang/String;)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method choose(Ljava/lang/String;)I
        // @declaration a static method of `NestedStringSwitchAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 28 37 39 60 62 96 105 111 120 123 152 154 156
        // local 4 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }
}
