// jarde: presentation of `IntArrayForeach` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class IntArrayForeach extends java.lang.Object {
    IntArrayForeach() {
        // @method <init>()V
        // @declaration a constructor of `IntArrayForeach`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int sum(int[] arg0) {
        // @method sum([I)I
        // @declaration a static method of `IntArrayForeach`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int[] local2;
        int local3;
        int local4;
        local1 = 0;
        local2 = arg0;
        local3 = local2.length;
        local4 = 0;
        while (local4 < local3) {
            int local5 = local2[local4];
            local1 = local1 + local5;
            local4 = local4 + 1;
        }
        return local1;
    }

    static int sumFrom(java.util.function.Supplier arg0) {
        // @method sumFrom(Ljava/util/function/Supplier;)I
        // @declaration a static method of `IntArrayForeach`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int[] local2;
        int local3;
        int local4;
        local1 = 0;
        local2 = (int[]) arg0.get();
        local3 = local2.length;
        local4 = 0;
        while (local4 < local3) {
            int local5 = local2[local4];
            local1 = local1 + local5;
            local4 = local4 + 1;
        }
        return local1;
    }
}
