// jarde: presentation of `cf11/UpstreamNestedLoops4` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class UpstreamNestedLoops4 extends java.lang.Object {
    public UpstreamNestedLoops4() {
        // @method <init>()V
        // @declaration a constructor of `cf11.UpstreamNestedLoops4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public int testFor() {
        // @method testFor()I
        // @declaration an instance method of `cf11.UpstreamNestedLoops4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        int tmp;
        int i;
        tmp = 1;
        i = 10;
        while (i > -1) {
            if (i > tmp) {
                int j;
                j = 0;
                while (j < 54) {
                    if (i < j) {
                        int k;
                        for (k = j; k < j + 4; k = k + 1) {
                            if (tmp > k) {
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
}
