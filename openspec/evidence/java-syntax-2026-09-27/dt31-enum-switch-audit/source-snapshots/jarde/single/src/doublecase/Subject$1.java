// jarde: presentation of `doublecase/Subject$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package doublecase;

class Subject$1 extends java.lang.Object {
    static final int[] $SwitchMap$doublecase$Count;

    static final int[] $SwitchMap$doublecase$Animal;

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `doublecase.Subject$1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        $SwitchMap$doublecase$Animal = new int[doublecase.Animal.values().length];
        try {
            doublecase.Subject$1.$SwitchMap$doublecase$Animal[doublecase.Animal.CAT.ordinal()] = 1;
        } catch (java.lang.NoSuchFieldError local0) {
        }
        try {
            doublecase.Subject$1.$SwitchMap$doublecase$Animal[doublecase.Animal.DOG.ordinal()] = 2;
        } catch (java.lang.NoSuchFieldError local0) {
        }
        $SwitchMap$doublecase$Count = new int[doublecase.Count.values().length];
        try {
            doublecase.Subject$1.$SwitchMap$doublecase$Count[doublecase.Count.ONE.ordinal()] = 1;
        } catch (java.lang.NoSuchFieldError local0) {
        }
        try {
            doublecase.Subject$1.$SwitchMap$doublecase$Count[doublecase.Count.TWO.ordinal()] = 2;
        } catch (java.lang.NoSuchFieldError local0) {
        }
    }
}
