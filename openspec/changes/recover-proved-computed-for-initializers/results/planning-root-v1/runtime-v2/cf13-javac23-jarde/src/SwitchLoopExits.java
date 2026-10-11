// jarde: presentation of `cf13/SwitchLoopExits` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf13;

public class SwitchLoopExits extends java.lang.Object {
    public SwitchLoopExits() {
        // @method <init>()V
        // @declaration a constructor of `cf13.SwitchLoopExits`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int walk(int n) {
        // @method walk(I)I
        // @declaration a static method of `cf13.SwitchLoopExits`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int score;
        int i;
        score = 0;
        for (i = 0; i < n; i = i + 1) {
            switch (i % 3) {
                case 0:
                    score = score + 1;
                    break;
                case 1:
                    score = score + 10;
                    continue;
                default:
                    score = score + 2;
                    break;
            }
            score = score + 3;
        }
        return score;
    }
}
