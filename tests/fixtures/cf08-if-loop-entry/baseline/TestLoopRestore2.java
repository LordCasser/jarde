// jarde: presentation of `loops/TestLoopRestore2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package loops;

public class TestLoopRestore2 extends java.lang.Object {
    public void test(int[] arg1, int[] arg2) {
        // @method test([I[I)V
        // @declaration an instance method of `loops.TestLoopRestore2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        int local3;
        local3 = 0;
        if (false) {
            do {
            } while (!true);
        }
        local3 = local3 + 1;
        return;
    }
}
