// jarde: presentation of `AssertVariants` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class AssertVariants extends java.lang.Object {
    static int effects;

    static final boolean $assertionsDisabled;

    public AssertVariants() {
        // @method <init>()V
        // @declaration a constructor of `AssertVariants`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static boolean probe(boolean value) {
        // @method probe(Z)Z
        // @declaration a static method of `AssertVariants`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        AssertVariants.effects = AssertVariants.effects * 10 + 2;
        return value;
    }

    private static java.lang.String detail() {
        // @method detail()Ljava/lang/String;
        // @declaration a static method of `AssertVariants`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        AssertVariants.effects = AssertVariants.effects * 10 + 3;
        return "message";
    }

    static int noMessage(boolean condition) {
        // @method noMessage(Z)I
        // @declaration a static method of `AssertVariants`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (!AssertVariants.$assertionsDisabled) {
            if (!probe(condition)) {
                throw new java.lang.AssertionError();
            }
        }
        return AssertVariants.effects;
    }

    static int multiple(boolean first, boolean second) {
        // @method multiple(ZZ)I
        // @declaration a static method of `AssertVariants`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (!AssertVariants.$assertionsDisabled) {
            if (!probe(first)) {
                throw new java.lang.AssertionError();
            }
        }
        if (!AssertVariants.$assertionsDisabled) {
            if (!probe(second)) {
                throw new java.lang.AssertionError((java.lang.Object) detail());
            }
        }
        return AssertVariants.effects;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `AssertVariants`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        $assertionsDisabled = (!AssertVariants.class.desiredAssertionStatus() ? 1 : 0) % 2 != 0;
        AssertVariants.effects = AssertVariants.effects + 1;
    }
}
