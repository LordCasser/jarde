// jarde: presentation of `PriorAssertIntArray` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class PriorAssertIntArray extends java.lang.Object {
    static final int VALUE = 7;

    static final boolean $assertionsDisabled;

    public PriorAssertIntArray() {
        // @method <init>()V
        // @declaration a constructor of `PriorAssertIntArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int[] value(boolean ok) {
        // @method value(Z)[I
        // @declaration a static method of `PriorAssertIntArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (!PriorAssertIntArray.$assertionsDisabled) {
            if (!ok) {
                throw new java.lang.AssertionError();
            }
        }
        return new int[]{VALUE};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `PriorAssertIntArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        $assertionsDisabled = (!PriorAssertIntArray.class.desiredAssertionStatus() ? 1 : 0) % 2 != 0;
    }
}
