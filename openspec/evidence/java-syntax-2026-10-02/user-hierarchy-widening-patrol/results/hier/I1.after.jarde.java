// jarde: presentation of `I1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class I1 extends java.lang.Object {
    public I1() {
        // @method <init>()V
        // @declaration a constructor of `I1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String useDefault() {
        // @method useDefault()Ljava/lang/String;
        // @declaration a static method of `I1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new I1$1().hello("d");
    }

    public static java.lang.String useOverride() {
        // @method useOverride()Ljava/lang/String;
        // @declaration a static method of `I1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new I1$En().hello("d");
    }

    public static java.lang.String useStatic() {
        // @method useStatic()Ljava/lang/String;
        // @declaration a static method of `I1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return I1$Greet.of().name();
    }

    public static java.lang.String viaInterface(I1$Greet arg0, java.lang.String arg1) {
        // @method viaInterface(LI1$Greet;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `I1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.hello(arg1);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `I1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) useDefault());
        java.lang.System.out.println((java.lang.String) useOverride());
        java.lang.System.out.println((java.lang.String) useStatic());
        java.lang.System.out.println((java.lang.String) viaInterface((I1$Greet) new I1$En(), "v"));
        return;
    }
}
