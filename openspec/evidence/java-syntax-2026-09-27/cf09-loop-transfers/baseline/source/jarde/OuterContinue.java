// jarde: presentation of `OuterContinue` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class OuterContinue extends java.lang.Object {
    OuterContinue() {
        // @method <init>()V
        // @declaration a constructor of `OuterContinue`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int run(int arg0) {
        // @method run(I)I
        // @declaration a static method of `OuterContinue`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        int local3;
        jarde_loop_4: for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            local3 = 0;
            while (local3 < arg0) {
                if (local3 == 2) {
                    continue jarde_loop_4;
                } else {
                    local1 = local1 + 1;
                    local3 = local3 + 1;
                }
            }
            local1 = local1 + 100;
        }
        return local1;
    }
}
