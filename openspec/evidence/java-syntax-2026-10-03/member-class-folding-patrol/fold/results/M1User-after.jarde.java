// jarde: presentation of `M1User` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class M1User extends M1$Base implements M1$Ctrl {
    M1User() {
        // @method <init>()V
        // @declaration a constructor of `M1User`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void run() {
        // @method run()V
        // @declaration a static method of `M1User`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        new M1User().hi();
        return;
    }
}
