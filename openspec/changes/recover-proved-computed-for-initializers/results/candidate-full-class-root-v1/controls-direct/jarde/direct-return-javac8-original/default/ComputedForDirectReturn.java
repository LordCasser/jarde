// jarde: presentation of `ComputedForDirectReturn` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class ComputedForDirectReturn extends java.lang.Object {
    static int end;

    static int calls;

    ComputedForDirectReturn() {
        // @method <init>()V
        // @declaration a constructor of `ComputedForDirectReturn`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int limit() {
        // @method limit()I
        // @declaration a static method of `ComputedForDirectReturn`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ComputedForDirectReturn.calls = ComputedForDirectReturn.calls + 1;
        return ComputedForDirectReturn.end;
    }

    static int find(int[] arg0, int arg1, int arg2) {
        // @method find([III)I
        // @declaration a static method of `ComputedForDirectReturn`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local3;
        for (local3 = limit() - 1; local3 >= arg2; local3 = local3 - 1) {
            if (arg0[local3] == arg1) {
                return local3;
            }
        }
        return -1;
    }
}
