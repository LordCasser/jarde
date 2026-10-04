// jarde: presentation of `DoubleBase` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
abstract class DoubleBase extends java.lang.Object {
    DoubleBase() {
        // @method <init>()V
        // @declaration a constructor of `DoubleBase`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: no body: the member `value()I` is declared abstract and its declaration carries no Code attribute
    abstract int value();
}
