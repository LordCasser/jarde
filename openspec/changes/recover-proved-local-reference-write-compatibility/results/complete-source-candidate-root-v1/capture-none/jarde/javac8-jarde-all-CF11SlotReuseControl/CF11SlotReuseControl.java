// jarde: presentation of `cf11/CF11SlotReuseControl` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public final class CF11SlotReuseControl extends java.lang.Object {
    public CF11SlotReuseControl() {
        // @method <init>()V
        // @declaration a constructor of `cf11.CF11SlotReuseControl`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Object afterStringLifetime(boolean arg0, java.util.List arg1) {
        // @method afterStringLifetime(ZLjava/util/List;)Ljava/lang/Object;
        // @declaration a static method of `cf11.CF11SlotReuseControl`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // local 2 is treated as one source variable, but the selected type `java.lang.String` is not proved compatible with the value written at BCI 22 (`java.lang.Object`); this region is refused instead of publishing a contradictory local (first write at BCI 2)
        // @bytecode 3 4 7
        // the statement at BCI 4 reads `local2`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
        // @bytecode 10
        // local 2 is treated as one source variable, but the selected type `java.lang.String` is not proved compatible with the value written at BCI 22 (`java.lang.Object`); this region is refused instead of publishing a contradictory local (first write at BCI 2)
        if (arg0) {
            // @bytecode 22
            // local 2 is treated as one source variable, but the selected type `java.lang.String` is not proved compatible with the value written at BCI 22 (`java.lang.Object`); this region is refused instead of publishing a contradictory local (first write at BCI 2)
        }
        // @bytecode 23 24
        // the statement at BCI 24 reads `local2`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
    }
}
