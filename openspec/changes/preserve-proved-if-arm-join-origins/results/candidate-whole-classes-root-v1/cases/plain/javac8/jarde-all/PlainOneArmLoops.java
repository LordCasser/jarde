// jarde: presentation of `PlainOneArmLoops` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class PlainOneArmLoops extends java.lang.Object {
    public PlainOneArmLoops() {
        // @method <init>()V
        // @declaration a constructor of `PlainOneArmLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int prefixWhile(boolean arg0, int arg1) {
        // @method prefixWhile(ZI)I
        // @declaration a static method of `PlainOneArmLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        local2 = 0;
        if (arg0) {
            int local3;
            for (local3 = 0; local3 < arg1; local3 = local3 + 1) {
                local2 = local2 + local3;
            }
        }
        return local2;
    }

    public static int noPrefix(boolean arg0, int arg1) {
        // @method noPrefix(ZI)I
        // @declaration a static method of `PlainOneArmLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        local2 = 0;
        if (arg0) {
            while (local2 < arg1) {
                local2 = local2 + 1;
            }
        }
        return local2;
    }

    public static int loopAndTail(boolean arg0, int arg1) {
        // @method loopAndTail(ZI)I
        // @declaration a static method of `PlainOneArmLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        local2 = 0;
        if (arg0) {
            int local3;
            for (local3 = 0; local3 < arg1; local3 = local3 + 1) {
                local2 = local2 + local3;
            }
            local2 = local2 + 100;
        }
        return local2;
    }

    public static int takenArm(boolean arg0, int arg1) {
        // @method takenArm(ZI)I
        // @declaration a static method of `PlainOneArmLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        local2 = 0;
        if (!arg0) {
            int local3;
            for (local3 = 0; local3 < arg1; local3 = local3 + 1) {
                local2 = local2 + local3;
            }
        }
        return local2;
    }
}
