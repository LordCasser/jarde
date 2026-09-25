// jarde: presentation of `DirectArrayCtorRef` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class DirectArrayCtorRef extends java.lang.Object {
    public DirectArrayCtorRef() {
        // @method <init>()V
        // @declaration a constructor of `DirectArrayCtorRef`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int[] make(int n) {
        // @method make(I)[I
        // @declaration a static method of `DirectArrayCtorRef`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (int[]) ((int p0) -> DirectArrayCtorRef.lambda$make$0(p0)).apply(n);
    }

    private static int[] lambda$make$0(int x$0) {
        // @method lambda$make$0(I)[I
        // @declaration a static method of `DirectArrayCtorRef`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new int[x$0];
    }
}
