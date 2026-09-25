// jarde: presentation of `BoundaryRunner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class BoundaryRunner extends java.lang.Object {
    BoundaryRunner() {
        // @method <init>()V
        // @declaration a constructor of `BoundaryRunner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String lookup(java.lang.String arg0) {
        // @method lookup(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `BoundaryRunner`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            // @bytecode 0
            // block at BCI 0 leaves through exception handler 0: a handler's shape is not part of the recoverable subset
        } catch (java.lang.Throwable local1) {
            return local1.getClass().getSimpleName();
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `BoundaryRunner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Measure[] local1 = Measure.values();
        java.lang.StringBuilder local2 = new java.lang.StringBuilder();
        Measure[] local3 = local1;
        int local4 = local3.length;
        int local5 = 0;
        // @bytecode 21 28 41
        // the loop whose header is the block at BCI 21 has a test, an exit or a latch this subset does not prove
        // @bytecode 90 48
        // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [90, 48]
    }
}
