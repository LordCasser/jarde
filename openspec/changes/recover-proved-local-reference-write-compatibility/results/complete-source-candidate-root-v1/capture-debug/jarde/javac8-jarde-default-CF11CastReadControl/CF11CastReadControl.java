// jarde: presentation of `cf11/CF11CastReadControl` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public final class CF11CastReadControl extends java.lang.Object {
    public CF11CastReadControl() {
        // @method <init>()V
        // @declaration a constructor of `cf11.CF11CastReadControl`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int castRead(java.util.List values) {
        // @method castRead(Ljava/util/List;)I
        // @declaration a static method of `cf11.CF11CastReadControl`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // local 1 is treated as one source variable, but the selected type `java.lang.String` is not proved compatible with the value written at BCI 19 (`java.lang.Object`); this region is refused instead of publishing a contradictory local (first write at BCI 2)
        if (!values.isEmpty()) {
            // @bytecode 19
            // local 1 is treated as one source variable, but the selected type `java.lang.String` is not proved compatible with the value written at BCI 19 (`java.lang.Object`); this region is refused instead of publishing a contradictory local (first write at BCI 2)
        }
        // @bytecode 20 21 24 27
        // the statement at BCI 27 reads `target`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
    }
}
