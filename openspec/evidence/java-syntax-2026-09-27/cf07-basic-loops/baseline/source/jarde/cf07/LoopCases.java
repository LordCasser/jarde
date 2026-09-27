// jarde: presentation of `cf07/LoopCases` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf07;

public class LoopCases extends java.lang.Object {
    public LoopCases() {
        // @method <init>()V
        // @declaration a constructor of `cf07.LoopCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int andWhile(boolean arg0) {
        // @method andWhile(Z)I
        // @declaration a static method of `cf07.LoopCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = 0;
        while (arg0 && local1 < 10) {
            local1 = local1 + 1;
        }
        return local1;
    }

    public static int counted(int arg0, int arg1) {
        // @method counted(II)I
        // @declaration a static method of `cf07.LoopCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        int local3;
        local2 = arg0 + arg1;
        local3 = arg0;
        while (local3 < arg1) {
            if (local3 == 7) {
                local2 = local2 + 2;
            } else {
                local2 = local2 * 2;
            }
            local3 = local3 + 1;
        }
        local2 = local2 - 1;
        return local2;
    }

    public static int lastIndexOf(int[] arg0, int arg1, int arg2, int arg3) {
        // jarde: not recovered: the recovery run for `lastIndexOf([IIII)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method lastIndexOf([IIII)I
        // @declaration a static method of `cf07.LoopCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 5 11 19 22 28
        // local 4 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }
}
