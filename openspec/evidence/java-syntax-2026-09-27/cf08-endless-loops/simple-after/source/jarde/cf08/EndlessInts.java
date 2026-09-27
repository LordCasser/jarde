// jarde: presentation of `cf08/EndlessInts` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf08;

public final class EndlessInts extends java.lang.Object {
    public EndlessInts() {
        // @method <init>()V
        // @declaration a constructor of `cf08.EndlessInts`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int find(int arg0) {
        // @method find(I)I
        // @declaration a static method of `cf08.EndlessInts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = 0;
        while (local1 < arg0) {
            if (local1 == 3) {
                break;
            } else {
                local1 = local1 + 1;
            }
        }
        return local1;
    }
}
