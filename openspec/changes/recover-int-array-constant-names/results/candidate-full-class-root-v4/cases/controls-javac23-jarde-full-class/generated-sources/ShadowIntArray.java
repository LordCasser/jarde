// jarde: presentation of `ShadowIntArray` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ShadowIntArray extends java.lang.Object {
    static final int VALUE = 7;

    public ShadowIntArray() {
        // @method <init>()V
        // @declaration a constructor of `ShadowIntArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int[] parameter(int VALUE) {
        // @method parameter(I)[I
        // @declaration a static method of `ShadowIntArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[]{7, VALUE};
    }

    static int[] local() {
        // @method local()[I
        // @declaration a static method of `ShadowIntArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int VALUE = 8;
        return new int[]{7, VALUE};
    }
}
