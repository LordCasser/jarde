// jarde: presentation of `single/Subject$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package single;

class Subject$1 extends java.lang.Object {
    static final int[] $SwitchMap$single$Mode;

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `single.Subject$1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        $SwitchMap$single$Mode = new int[single.Mode.values().length];
        try {
            single.Subject$1.$SwitchMap$single$Mode[single.Mode.ONE.ordinal()] = 1;
        } catch (java.lang.NoSuchFieldError local0) {
        }
        try {
            single.Subject$1.$SwitchMap$single$Mode[single.Mode.TWO.ordinal()] = 2;
        } catch (java.lang.NoSuchFieldError local0) {
        }
    }
}
