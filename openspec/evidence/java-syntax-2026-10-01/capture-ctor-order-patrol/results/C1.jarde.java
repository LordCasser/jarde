// jarde: presentation of `C1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class C1 extends java.lang.Object {
    public C1() {
        // @method <init>()V
        // @declaration a constructor of `C1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int captureLocal(int arg0) {
        // @method captureLocal(I)I
        // @declaration a static method of `C1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        C1$1 local1 = new C1$1(arg0);
        return local1.apply(5);
    }

    public static int captureEffectivelyFinal() {
        // @method captureEffectivelyFinal()I
        // @declaration a static method of `C1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local0;
        int local1;
        local0 = 0;
        for (local1 = 0; local1 < 3; local1 = local1 + 1) {
            int local2 = local1;
            local0 = local0 + new C1$2(local2).apply(local2);
        }
        return local0;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `C1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(captureLocal(10));
        java.lang.System.out.println(captureEffectivelyFinal());
        return;
    }
}
