// jarde: presentation of `DirectMethodRef` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class DirectMethodRef extends java.lang.Object {
    public DirectMethodRef() {
        // @method <init>()V
        // @declaration a constructor of `DirectMethodRef`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int abs(int n) {
        // @method abs(I)I
        // @declaration a static method of `DirectMethodRef`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return ((java.util.function.IntUnaryOperator) java.lang.Math::abs).applyAsInt(n);
    }
}
