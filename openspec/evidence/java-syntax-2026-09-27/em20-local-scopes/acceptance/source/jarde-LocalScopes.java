// jarde: presentation of `em20/LocalScopes` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em20;

public class LocalScopes extends java.lang.Object {
    public LocalScopes() {
        // @method <init>()V
        // @declaration a constructor of `em20.LocalScopes`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int joined(boolean arg0, int arg1) {
        // @method joined(ZI)I
        // @declaration a static method of `em20.LocalScopes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        if (arg0) {
            local2 = arg1 + 1;
        } else {
            local2 = arg1 - 1;
        }
        return local2;
    }

    public static int loop(int arg0) {
        // @method loop(I)I
        // @declaration a static method of `em20.LocalScopes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local3 = local2 + 1;
            local1 = local1 + local3;
        }
        return local1;
    }

    public static int synchronizedLoop(int arg0) {
        // @method synchronizedLoop(I)I
        // @declaration a static method of `em20.LocalScopes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        synchronized (em20.LocalScopes.class) {
            local1 = arg0 + 1;
        }
        // @bytecode 20
        // local 2 is treated as one source variable, but BCI 3 writes `Object` and BCI 20 writes `int`; no Java declaration can hold both, so this region is refused instead of publishing a contradictory local
        // @bytecode 22
        // local 3 is treated as one source variable, but BCI 14 writes `Object` and BCI 22 writes `int`; no Java declaration can hold both, so this region is refused instead of publishing a contradictory local
        // @bytecode 23 24 25 31 32
        // the statement at BCI 25 reads `local3`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
        // @bytecode 38 39
        // the statement at BCI 39 reads `local2`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
    }

    public static int caught(boolean arg0) {
        // @method caught(Z)I
        // @declaration a static method of `em20.LocalScopes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = 1;
        try {
            if (arg0) {
                throw new java.lang.IllegalArgumentException("requested");
            } else {
                local1 = 2;
            }
        } catch (java.lang.IllegalArgumentException local2) {
            local1 = 3;
        }
        return local1;
    }
}
