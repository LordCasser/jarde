// jarde: presentation of `M1$Base` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class M1$Base extends java.lang.Object {
    M1$Base() {
        // @method <init>()V
        // @declaration a constructor of `M1$Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    void hi() {
        // @method hi()V
        // @declaration an instance method of `M1$Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("hi");
        return;
    }
}
