// jarde: presentation of `StringIterableForeach` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class StringIterableForeach extends java.lang.Object {
    StringIterableForeach() {
        // @method <init>()V
        // @declaration a constructor of `StringIterableForeach`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int sumLengths(java.lang.Iterable arg0) {
        // @method sumLengths(Ljava/lang/Iterable;)I
        // @declaration a static method of `StringIterableForeach`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1 = 0;
        java.util.Iterator local2 = arg0.iterator();
        // @bytecode 9
        // the loop@1 rule did not claim the block at BCI 9: it requires a test block whose every instruction is part of a value expression, and the instruction at BCI 10 is not part of one, so presenting the structure would have moved that effect out of the shape it decides
        // @bytecode 9 18 38
        // 3 live block(s) are reachable only through edges the normal-flow view leaves out: [9, 18, 38]
    }

    static int sumLengthsFrom(java.util.function.Supplier arg0) {
        // @method sumLengthsFrom(Ljava/util/function/Supplier;)I
        // @declaration a static method of `StringIterableForeach`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1 = 0;
        java.util.Iterator local2 = ((java.lang.Iterable) arg0.get()).iterator();
        // @bytecode 17
        // the loop@1 rule did not claim the block at BCI 17: it requires a test block whose every instruction is part of a value expression, and the instruction at BCI 18 is not part of one, so presenting the structure would have moved that effect out of the shape it decides
        // @bytecode 17 26 46
        // 3 live block(s) are reachable only through edges the normal-flow view leaves out: [17, 26, 46]
    }
}
