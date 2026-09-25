// jarde: presentation of `ArrayPostfixElement` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ArrayPostfixElement extends java.lang.Object {
    public ArrayPostfixElement() {
        // @method <init>()V
        // @declaration a constructor of `ArrayPostfixElement`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int[] make(int arg0) {
        // @method make(I)[I
        // @declaration a static method of `ArrayPostfixElement`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[]{1, arg0++, arg0 * 2};
    }
}
