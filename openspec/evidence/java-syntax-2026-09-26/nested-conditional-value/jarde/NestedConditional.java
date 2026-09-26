// jarde: presentation of `NestedConditional` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class NestedConditional extends java.lang.Object {
    private NestedConditional() {
        // @method <init>()V
        // @declaration a constructor of `NestedConditional`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int nested(int arg0) {
        // @method nested(I)I
        // @declaration a static method of `NestedConditional`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 > 10) {
            if (arg0 > 100) {
            }
        }
        // @bytecode 21
        // the value at BCI 21 is the entry state of stack depth 0, which no instruction produced
        // @bytecode 22 23
        // the statement at BCI 23 reads `local1`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
    }

    public static int nestedEffects(int arg0, java.lang.StringBuilder arg1) {
        // @method nestedEffects(ILjava/lang/StringBuilder;)I
        // @declaration a static method of `NestedConditional`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 > outerLimit(arg0, arg1)) {
            if (arg0 > innerLimit(arg0, arg1)) {
                // @bytecode 18 23
                // the saved producer at BCI 23 has no bounded final expression consumer
            }
        } else {
            // @bytecode 50 55
            // the saved producer at BCI 55 has no bounded final expression consumer
        }
        // @bytecode 58
        // the value at BCI 58 is the entry state of stack depth 0, which no instruction produced
        // @bytecode 59 60
        // the statement at BCI 60 reads `local2`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
        // @bytecode 44 47
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [44]
    }

    private static int outerLimit(int arg0, java.lang.StringBuilder arg1) {
        // @method outerLimit(ILjava/lang/StringBuilder;)I
        // @declaration a static method of `NestedConditional`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        arg1.append('O');
        if (arg0 == 0) {
            throw new java.lang.IllegalArgumentException("outer");
        } else {
            return 10;
        }
    }

    private static int innerLimit(int arg0, java.lang.StringBuilder arg1) {
        // @method innerLimit(ILjava/lang/StringBuilder;)I
        // @declaration a static method of `NestedConditional`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        arg1.append('I');
        if (arg0 == 50) {
            throw new java.lang.IllegalStateException("inner");
        } else {
            return 100;
        }
    }

    private static int arm(java.lang.StringBuilder arg0, java.lang.String arg1, int arg2, boolean arg3) {
        // @method arm(Ljava/lang/StringBuilder;Ljava/lang/String;IZ)I
        // @declaration a static method of `NestedConditional`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        arg0.append(arg1);
        if (arg3) {
            // @bytecode 10
            // the instruction at BCI 10 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
            // @bytecode 13
            // the instruction at BCI 13 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
            // @bytecode 36 33 26
            // the value at BCI 36 comes from an Duplicate at BCI 13, which produces no expression this subset writes
        } else {
            return arg2;
        }
    }
}
