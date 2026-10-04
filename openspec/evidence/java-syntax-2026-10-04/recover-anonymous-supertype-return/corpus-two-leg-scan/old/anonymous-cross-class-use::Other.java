// jarde: presentation of `Other` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class Other extends java.lang.Object {
    Other() {
        // @method <init>()V
        // @declaration a constructor of `Other`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static Base two() {
        // @method two()LBase;
        // @declaration a static method of `Other`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new Owner$1();
    }
}
