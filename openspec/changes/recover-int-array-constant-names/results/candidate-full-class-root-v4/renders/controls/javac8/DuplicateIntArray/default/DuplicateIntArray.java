// jarde: presentation of `DuplicateIntArray` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class DuplicateIntArray extends java.lang.Object {
    static final int VALUE = 7;

    static final int OTHER = 7;

    public DuplicateIntArray() {
        // @method <init>()V
        // @declaration a constructor of `DuplicateIntArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int[] values() {
        // @method values()[I
        // @declaration a static method of `DuplicateIntArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[]{7};
    }
}
