// jarde: presentation of `Grid` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Grid extends java.lang.Object {
    public Grid() {
        // @method <init>()V
        // @declaration a constructor of `Grid`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int nestedBreak(int arg0) {
        // @method nestedBreak(I)I
        // @declaration a static method of `Grid`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            local3 = 0;
            while (local3 < arg0) {
                if (local2 + local3 > 5) {
                    break;
                } else {
                    local1 = local1 + 1;
                    local3 = local3 + 1;
                }
            }
        }
        return local1;
    }

    static int labeledContinue(int arg0) {
        // @method labeledContinue(I)I
        // @declaration a static method of `Grid`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            local3 = 0;
            while (local3 < arg0) {
                if (local3 == 2) {
                    break;
                } else {
                    local1 = local1 + 1;
                    local3 = local3 + 1;
                }
            }
        }
        return local1;
    }

    static int labeledBreak(int arg0) {
        // @method labeledBreak(I)I
        // @declaration a static method of `Grid`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        local2 = 0;
        int local3;
        jarde_loop_4: while (local2 < arg0) {
            local3 = 0;
            while (local3 < arg0) {
                if (local3 == 3) {
                    break jarde_loop_4;
                } else {
                    local1 = local1 + 1;
                    local3 = local3 + 1;
                }
            }
            local1 = local1 + 10;
            local2 = local2 + 1;
        }
        return local1;
    }
}
