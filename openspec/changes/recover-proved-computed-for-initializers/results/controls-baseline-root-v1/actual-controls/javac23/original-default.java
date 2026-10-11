// jarde: presentation of `ComputedForControls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class ComputedForControls extends java.lang.Object {
    static int end;

    static int trace;

    static int mode;

    static final java.lang.RuntimeException FAILURE;

    ComputedForControls() {
        // @method <init>()V
        // @declaration a constructor of `ComputedForControls`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int f() {
        // @method f()I
        // @declaration a static method of `ComputedForControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ComputedForControls.trace = ComputedForControls.trace * 10 + 1;
        if (ComputedForControls.mode == 1) {
            throw ComputedForControls.FAILURE;
        } else {
            return ComputedForControls.end;
        }
    }

    static void g() {
        // @method g()V
        // @declaration a static method of `ComputedForControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ComputedForControls.trace = ComputedForControls.trace * 10 + 2;
        ComputedForControls.end = 2;
        if (ComputedForControls.mode == 2) {
            throw ComputedForControls.FAILURE;
        } else {
            return;
        }
    }

    static int find(int[] arg0, int arg1, int arg2) {
        // jarde: not recovered: the recovery run for `find([III)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method find([III)I
        // @declaration a static method of `ComputedForControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 9 14 21 23 29 31 39 44 46
        // local 3 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static int shared(int arg0) {
        // jarde: not recovered: the recovery run for `shared(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method shared(I)I
        // @declaration a static method of `ComputedForControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 4 0 3 5 6 7 8 11 12 13 16 19 22 23 25 26 27 28
        // the copy at BCI 3 has no proved local assignment
        // @bytecode 7
        // the copy at BCI 3 has no proved local assignment
    }

    static int observedAfter(int arg0) {
        // @method observedAfter(I)I
        // @declaration a static method of `ComputedForControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = f() - 1;
        while (local1 >= arg0) {
            local1 = local1 - 1;
        }
        return local1;
    }

    static int externalWrite(int arg0) {
        // @method externalWrite(I)I
        // @declaration a static method of `ComputedForControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = f() - 1;
        while (local1 >= arg0) {
            local1 = local1 - 1;
        }
        local1 = arg0;
        return local1;
    }

    static int scoped(int arg0, boolean arg1) {
        // jarde: not recovered: the recovery run for `scoped(IZ)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method scoped(IZ)I
        // @declaration a static method of `ComputedForControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 10 15 21 23
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `ComputedForControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        FAILURE = new java.lang.IllegalStateException("chosen");
    }
}
