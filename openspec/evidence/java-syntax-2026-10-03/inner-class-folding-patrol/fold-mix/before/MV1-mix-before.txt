// jarde: presentation of `MV1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class MV1 extends java.lang.Object {
    public MV1() {
        // @method <init>()V
        // @declaration a constructor of `MV1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    int run() {
        // @method run()I
        // @declaration an instance method of `MV1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return new MV1$Inner(this).v();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `MV1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(new MV1().run());
        return;
    }
}
