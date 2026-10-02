// jarde: presentation of `VN3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class VN3 extends java.lang.Object {
    public VN3() {
        // @method <init>()V
        // @declaration a constructor of `VN3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static VN3$Op capture(int arg0) {
        // @method capture(I)LVN3$Op;
        // @declaration a static method of `VN3`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new VN3$1(arg0);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `VN3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(capture(5).apply(37));
        return;
    }
}
