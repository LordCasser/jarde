// jarde: presentation of `cf11/NestedLoopSideExitControls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class NestedLoopSideExitControls extends java.lang.Object {
    public NestedLoopSideExitControls() {
        // @method <init>()V
        // @declaration a constructor of `cf11.NestedLoopSideExitControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public int testFor(int returnThreshold, int bound, int window) {
        // @method testFor(III)I
        // @declaration an instance method of `cf11.NestedLoopSideExitControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        int tmp;
        int i;
        tmp = 1;
        i = 10;
        while (i > -1) {
            if (i > tmp) {
                int j;
                j = 0;
                while (j < bound) {
                    if (i < j) {
                        int k;
                        for (k = j; k < j + window; k = k + 1) {
                            if (returnThreshold > k) {
                                return 0;
                            }
                        }
                        break;
                    }
                    j = j + 4;
                }
            }
            tmp = tmp + 1;
            i = i - 1;
        }
        return tmp;
    }

    public int runCounts(int returnThreshold, int bound, int window) {
        // @method runCounts(III)I
        // @declaration an instance method of `cf11.NestedLoopSideExitControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        int tmp;
        int ticks;
        int i;
        tmp = 1;
        ticks = 0;
        i = 10;
        while (i > -1) {
            if (i > tmp) {
                int j;
                j = 0;
                while (j < bound) {
                    if (i < j) {
                        int k;
                        for (k = j; k < j + window; k = k + 1) {
                            ticks = ticks + 1;
                            if (returnThreshold > k) {
                                return ticks;
                            }
                        }
                        break;
                    }
                    j = j + 4;
                }
            }
            tmp = tmp + 1;
            i = i - 1;
        }
        return tmp * 1000 + ticks;
    }
}
