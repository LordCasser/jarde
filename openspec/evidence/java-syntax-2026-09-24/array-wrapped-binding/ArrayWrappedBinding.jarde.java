// jarde: presentation of `ArrayWrappedBinding` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class ArrayWrappedBinding extends java.lang.Object {
    static int calls;

    static int[] watched;

    static java.lang.RuntimeException failure;

    static int last;

    ArrayWrappedBinding() {
        // @method <init>()V
        // @declaration a constructor of `ArrayWrappedBinding`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int tick() {
        // @method tick()I
        // @declaration a static method of `ArrayWrappedBinding`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ArrayWrappedBinding.calls = ArrayWrappedBinding.calls + 1;
        if (ArrayWrappedBinding.failure != null) {
            throw ArrayWrappedBinding.failure;
        } else {
            return ArrayWrappedBinding.calls;
        }
    }

    static int mutateAndTick() {
        // @method mutateAndTick()I
        // @declaration a static method of `ArrayWrappedBinding`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ArrayWrappedBinding.calls = ArrayWrappedBinding.calls + 1;
        ArrayWrappedBinding.watched[0] = 90 + ArrayWrappedBinding.calls;
        if (ArrayWrappedBinding.failure != null) {
            throw ArrayWrappedBinding.failure;
        } else {
            return ArrayWrappedBinding.calls;
        }
    }

    static int plusArrayFirst(int[] arg0) {
        // @method plusArrayFirst([I)I
        // @declaration a static method of `ArrayWrappedBinding`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int[] local2;
        int local3;
        int local4;
        local1 = 0;
        local2 = arg0;
        local3 = local2.length;
        for (local4 = 0; local4 < local3; local4 = local4 + 1) {
            int local5 = local2[local4] + tick();
            local1 = local1 + local5;
        }
        return local1;
    }

    static int plusTickFirst(int[] arg0) {
        // @method plusTickFirst([I)I
        // @declaration a static method of `ArrayWrappedBinding`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int[] local2;
        int local3;
        int local4;
        local1 = 0;
        local2 = arg0;
        ArrayWrappedBinding.watched = arg0;
        local3 = local2.length;
        for (local4 = 0; local4 < local3; local4 = local4 + 1) {
            int local5 = mutateAndTick() + local2[local4];
            local1 = local1 + local5;
        }
        return local1;
    }

    static int callTickThenArray(int[] arg0) {
        // @method callTickThenArray([I)I
        // @declaration a static method of `ArrayWrappedBinding`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int[] local2;
        int local3;
        int local4;
        local1 = 0;
        local2 = arg0;
        local3 = local2.length;
        for (local4 = 0; local4 < local3; local4 = local4 + 1) {
            consume(tick(), local2[local4]);
            local1 = local1 + ArrayWrappedBinding.last;
        }
        return local1;
    }

    static int callMutateThenArray(int[] arg0) {
        // @method callMutateThenArray([I)I
        // @declaration a static method of `ArrayWrappedBinding`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int[] local2;
        int local3;
        int local4;
        local1 = 0;
        ArrayWrappedBinding.watched = arg0;
        local2 = arg0;
        local3 = local2.length;
        for (local4 = 0; local4 < local3; local4 = local4 + 1) {
            consume(mutateAndTick(), local2[local4]);
            local1 = local1 + ArrayWrappedBinding.last;
        }
        return local1;
    }

    static int readAfterEffect(int[] arg0) {
        // @method readAfterEffect([I)I
        // @declaration a static method of `ArrayWrappedBinding`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int[] local2;
        int local3;
        int local4;
        local1 = 0;
        ArrayWrappedBinding.watched = arg0;
        local2 = arg0;
        local3 = local2.length;
        for (local4 = 0; local4 < local3; local4 = local4 + 1) {
            mutateAndTick();
            int local5 = local2[local4];
            local1 = local1 + local5;
        }
        return local1;
    }

    static int twoReadsWithMutation(int[] arg0) {
        // @method twoReadsWithMutation([I)I
        // @declaration a static method of `ArrayWrappedBinding`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int[] local2;
        int local3;
        int local4;
        local1 = 0;
        ArrayWrappedBinding.watched = arg0;
        local2 = arg0;
        local3 = local2.length;
        for (local4 = 0; local4 < local3; local4 = local4 + 1) {
            int local5 = local2[local4];
            mutateAndTick();
            int local6 = local2[local4];
            local1 = local1 + (local5 + local6);
        }
        return local1;
    }

    static int wrappedOnlyRead(int[] arg0) {
        // @method wrappedOnlyRead([I)I
        // @declaration a static method of `ArrayWrappedBinding`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int[] local2;
        int local3;
        int local4;
        local1 = 0;
        local2 = arg0;
        local3 = local2.length;
        for (local4 = 0; local4 < local3; local4 = local4 + 1) {
            local1 = local1 + (local2[local4] + tick());
        }
        return local1;
    }

    static void consume(int arg0, int arg1) {
        // @method consume(II)V
        // @declaration a static method of `ArrayWrappedBinding`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ArrayWrappedBinding.last = arg0 * 1000 + arg1;
        return;
    }
}
