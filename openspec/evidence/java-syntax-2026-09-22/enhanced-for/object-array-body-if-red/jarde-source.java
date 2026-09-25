// jarde: presentation of `ObjectArrayForeach` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class ObjectArrayForeach extends java.lang.Object {
    ObjectArrayForeach() {
        // @method <init>()V
        // @declaration a constructor of `ObjectArrayForeach`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int sumHash(java.lang.Object[] arg0) {
        // @method sumHash([Ljava/lang/Object;)I
        // @declaration a static method of `ObjectArrayForeach`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1 = 0;
        java.lang.Object[] local2 = arg0;
        int local3 = local2.length;
        int local4 = 0;
        // @bytecode 10 16 27
        // the loop whose header is the block at BCI 10 has a test, an exit or a latch this subset does not prove
        // @bytecode 41 35
        // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [41, 35]
    }

    static int sumHashFrom(java.util.function.Supplier arg0) {
        // @method sumHashFrom(Ljava/util/function/Supplier;)I
        // @declaration a static method of `ObjectArrayForeach`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1 = 0;
        java.lang.Object[] local2 = (java.lang.Object[]) arg0.get();
        int local3 = local2.length;
        int local4 = 0;
        // @bytecode 18 24 35
        // the loop whose header is the block at BCI 18 has a test, an exit or a latch this subset does not prove
        // @bytecode 49 43
        // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [49, 43]
    }
}
