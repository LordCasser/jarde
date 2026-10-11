// jarde: presentation of `SharedStoreInput` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class SharedStoreInput extends java.lang.Object {
    static int savedField;

    static int limitCalls;

    static int limitValue;

    static boolean throwFromLimit;

    SharedStoreInput() {
        // @method <init>()V
        // @declaration a constructor of `SharedStoreInput`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int limit() {
        // @method limit()I
        // @declaration a static method of `SharedStoreInput`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        SharedStoreInput.limitCalls = SharedStoreInput.limitCalls + 1;
        if (SharedStoreInput.throwFromLimit) {
            throw new java.lang.IllegalStateException("limit");
        } else {
            return SharedStoreInput.limitValue;
        }
    }

    static int find(int[] arg0, int arg1, int arg2) {
        // jarde: not recovered: the recovery run for `find([III)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method find([III)I
        // @declaration a static method of `SharedStoreInput`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 5 0
        // the instruction at BCI 5 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6 0 3 4 5 9 10 11 12 15 16 17 18 19 22 23 24 27 30 31
        // the copy at BCI 5 has no proved local assignment
        // @bytecode 9 0
        // the copy at BCI 5 has no proved local assignment
        // @bytecode 10 15 22 24 12
        // the counted loop's preceding local initialisation did not produce one Java header clause
    }
}
