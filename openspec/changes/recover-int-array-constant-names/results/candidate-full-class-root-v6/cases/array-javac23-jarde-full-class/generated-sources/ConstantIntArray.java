// jarde: presentation of `ConstantIntArray` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ConstantIntArray extends java.lang.Object {
    public static final int CONST_INT = 65535;

    public ConstantIntArray() {
        // @method <init>()V
        // @declaration a constructor of `ConstantIntArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public int[] test() {
        // @method test()[I
        // @declaration an instance method of `ConstantIntArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return new int[]{127, 129, CONST_INT};
    }
}
