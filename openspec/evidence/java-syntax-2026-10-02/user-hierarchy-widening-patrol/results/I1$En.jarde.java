// jarde: presentation of `I1$En` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class I1$En extends java.lang.Object implements I1$Greet {
    I1$En() {
        // @method <init>()V
        // @declaration a constructor of `I1$En`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.String name() {
        // @method name()Ljava/lang/String;
        // @declaration an instance method of `I1$En`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return "en";
    }

    public java.lang.String hello(java.lang.String arg1) {
        // @method hello(Ljava/lang/String;)Ljava/lang/String;
        // @declaration an instance method of `I1$En`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return "hello:" + arg1;
    }
}
