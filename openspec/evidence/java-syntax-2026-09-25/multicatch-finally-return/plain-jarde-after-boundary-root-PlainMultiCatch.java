// jarde: presentation of `PlainMultiCatch` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class PlainMultiCatch extends java.lang.Object {
    public PlainMultiCatch() {
        // @method <init>()V
        // @declaration a constructor of `PlainMultiCatch`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String choose(int arg0) {
        // @method choose(I)Ljava/lang/String;
        // @declaration a static method of `PlainMultiCatch`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (arg0 == 1) {
                throw new java.lang.IllegalArgumentException("a");
            } else {
                if (arg0 == 2) {
                    throw new java.lang.IllegalStateException("s");
                } else {
                    return "ok";
                }
            }
        } catch (java.lang.IllegalArgumentException | java.lang.IllegalStateException local1) {
            return local1.getClass().getSimpleName() + ":" + local1.getMessage();
        }
    }
}

