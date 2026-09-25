// jarde: presentation of `Probe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class Probe extends java.lang.Object implements Child {
    public Probe() {
        // @method <init>()V
        // @declaration a constructor of `Probe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public int value() {
        // @method value()I
        // @declaration an instance method of `Probe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return Child.super.value();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(new Probe().value());
        return;
    }
}
