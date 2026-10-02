// jarde: presentation of `F1$Diamond` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class F1$Diamond extends java.lang.Object implements F1$A, F1$B {
    F1$Diamond() {
        // @method <init>()V
        // @declaration a constructor of `F1$Diamond`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.String name() {
        // @method name()Ljava/lang/String;
        // @declaration an instance method of `F1$Diamond`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return F1$A.super.name() + F1$B.super.name();
    }
}
