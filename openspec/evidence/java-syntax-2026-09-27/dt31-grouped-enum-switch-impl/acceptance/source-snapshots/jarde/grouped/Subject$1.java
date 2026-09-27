// jarde: presentation of `grouped/Subject$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package grouped;

class Subject$1 extends java.lang.Object {
    static final int[] $SwitchMap$grouped$Count;

    static final int[] $SwitchMap$grouped$Animal;

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `grouped.Subject$1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        $SwitchMap$grouped$Animal = new int[grouped.Animal.values().length];
        try {
            grouped.Subject$1.$SwitchMap$grouped$Animal[grouped.Animal.CAT.ordinal()] = 1;
        } catch (java.lang.NoSuchFieldError local0) {
        }
        try {
            grouped.Subject$1.$SwitchMap$grouped$Animal[grouped.Animal.DOG.ordinal()] = 2;
        } catch (java.lang.NoSuchFieldError local0) {
        }
        $SwitchMap$grouped$Count = new int[grouped.Count.values().length];
        try {
            grouped.Subject$1.$SwitchMap$grouped$Count[grouped.Count.ONE.ordinal()] = 1;
        } catch (java.lang.NoSuchFieldError local0) {
        }
        try {
            grouped.Subject$1.$SwitchMap$grouped$Count[grouped.Count.TWO.ordinal()] = 2;
        } catch (java.lang.NoSuchFieldError local0) {
        }
    }
}
