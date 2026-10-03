// jarde: presentation of `WCMulti` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class WCMulti extends java.lang.Object {
    static WCMulti$B held;

    public WCMulti() {
        // @method <init>()V
        // @declaration a constructor of `WCMulti`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `WCMulti`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(WCMulti$A.sv() + WCMulti$B.sv());
        java.lang.System.out.println(WCMulti.held.iv());
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `WCMulti`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        WCMulti.held = new WCMulti$Impl();
    }
}
