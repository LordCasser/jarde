// jarde: presentation of `SlotReuseBoundaries` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class SlotReuseBoundaries extends java.lang.Object {
    public SlotReuseBoundaries() {
        // @method <init>()V
        // @declaration a constructor of `SlotReuseBoundaries`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int sameType(int arg0) {
        // @method sameType(I)I
        // @declaration a static method of `SlotReuseBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1 = 0;
        int local2 = arg0 + 1;
        local1 = local1 + local2;
        local2 = arg0 + 2;
        local1 = local1 + local2;
        return local1;
    }

    public static int exclusiveBranch(boolean arg0) {
        // @method exclusiveBranch(Z)I
        // @declaration a static method of `SlotReuseBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        if (arg0) {
            int[] local2 = new int[]{7};
            local1 = local2[0];
        } else {
            int local2_2 = 9;
            local1 = local2_2;
        }
        return local1;
    }

    public static int loopBodyReuse(int arg0) {
        // @method loopBodyReuse(I)I
        // @declaration a static method of `SlotReuseBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = 0;
        while (arg0 > 0) {
            // @bytecode 13
            // local 2 is treated as one source variable, but BCI 13 writes `int[]` and BCI 23 writes `int`; no Java declaration can hold both, so this region is refused instead of publishing a contradictory local
            // @bytecode 14 15 16 17 18 19
            // the statement at BCI 19 reads `local2`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
            // @bytecode 23
            // local 2 is treated as one source variable, but BCI 13 writes `int[]` and BCI 23 writes `int`; no Java declaration can hold both, so this region is refused instead of publishing a contradictory local
            // @bytecode 24 25 26 27
            // the statement at BCI 27 reads `local2`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
            arg0 = arg0 - 1;
        }
        return local1;
    }

    public static int handlerReuse(boolean arg0) {
        // @method handlerReuse(Z)I
        // @declaration a static method of `SlotReuseBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        // @bytecode 0 20 28 31
        // local 2 escapes catch parameter scope at region [0, 1]; its catch header cannot declare a method-visible local
        return local1;
    }

    public static long category2Adjacent() {
        // jarde: not recovered: the recovery run for `category2Adjacent()J` produced no statement (explanation only); the artifact's own comment lines are below
        // @method category2Adjacent()J
        // @declaration a static method of `SlotReuseBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 7
        // local 0 is treated as one source variable, but BCI 7 writes `int[]` and BCI 19 writes `long`; no Java declaration can hold both, so this region is refused instead of publishing a contradictory local
        // @bytecode 8 9 10 11 14 15 19 21 22 23 24 25 26 27
        // the statement at BCI 11 reads `local0`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
    }
}
