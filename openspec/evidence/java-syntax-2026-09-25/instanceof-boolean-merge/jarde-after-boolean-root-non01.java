// jarde: presentation of `BooleanMergeControls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class BooleanMergeControls extends java.lang.Object {
    static int calls;

    public BooleanMergeControls() {
        // @method <init>()V
        // @declaration a constructor of `BooleanMergeControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.Object value(java.lang.Object arg0) {
        // @method value(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration a static method of `BooleanMergeControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        BooleanMergeControls.calls = BooleanMergeControls.calls + 1;
        return arg0;
    }

    static boolean positive(java.lang.Object arg0) {
        // @method positive(Ljava/lang/Object;)Z
        // @declaration a static method of `BooleanMergeControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return (value(arg0) instanceof java.lang.String ? 2 : 3) % 2 != 0;
    }

    static boolean negative(java.lang.Object arg0) {
        // @method negative(Ljava/lang/Object;)Z
        // @declaration a static method of `BooleanMergeControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return !(value(arg0) instanceof java.lang.String);
    }
}

