// jarde: presentation of `JadxOrderingControl` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class JadxOrderingControl extends java.lang.Object {
    static int trace;

    public JadxOrderingControl() {
        // @method <init>()V
        // @declaration a constructor of `JadxOrderingControl`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int mark(int arg0) {
        // @method mark(I)I
        // @declaration a static method of `JadxOrderingControl`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        JadxOrderingControl.trace = JadxOrderingControl.trace * 10 + arg0;
        return arg0;
    }

    static int[] build() {
        // @method build()[I
        // @declaration a static method of `JadxOrderingControl`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int[] local0 = new int[2];
        local0[1] = mark(1);
        local0[0] = mark(2);
        return local0;
    }
}
