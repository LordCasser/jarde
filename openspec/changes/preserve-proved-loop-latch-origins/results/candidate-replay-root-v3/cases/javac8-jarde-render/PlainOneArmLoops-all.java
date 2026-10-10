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
        // jarde: not recovered: the recovery run for `prefixWhile(ZI)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method prefixWhile(ZI)I
        // @declaration a static method of `PlainOneArmLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 6 7 8 9 10 13 14 15 16 17 20 23 24
        // the arms of the branch in block 0 do not meet at one join
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
        // jarde: not recovered: the recovery run for `loopAndTail(ZI)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method loopAndTail(ZI)I
        // @declaration a static method of `PlainOneArmLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 6 7 8 9 10 13 14 15 16 17 20 23 26 27
        // the arms of the branch in block 0 do not meet at one join
    }

    public static int takenArm(boolean arg0, int arg1) {
        // jarde: not recovered: the recovery run for `takenArm(ZI)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method takenArm(ZI)I
        // @declaration a static method of `PlainOneArmLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 6 7 8 9 10 13 14 15 16 17 20 23 24
        // the arms of the branch in block 0 do not meet at one join
    }
}
