// jarde: presentation of `NonIterableCursor` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NonIterableCursor extends java.lang.Object {
    public NonIterableCursor() {
        // @method <init>()V
        // @declaration a constructor of `NonIterableCursor`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int sum(NonIterableCursor$CursorBox arg0) {
        // @method sum(LNonIterableCursor$CursorBox;)I
        // @declaration a static method of `NonIterableCursor`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        java.util.Iterator local2;
        local1 = 0;
        local2 = arg0.iterator();
        while (local2.hasNext()) {
            java.lang.String local3 = (java.lang.String) local2.next();
            local1 = local1 + local3.length();
        }
        return local1;
    }
}
