// jarde: presentation of `OrdinaryInit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class OrdinaryInit extends java.lang.Object {
    static int first = 4;

    static int second = twice();

    OrdinaryInit() {
        // @method <init>()V
        // @declaration a constructor of `OrdinaryInit`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int twice() {
        // @method twice()I
        // @declaration a static method of `OrdinaryInit`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return OrdinaryInit.first * 2;
    }
}
