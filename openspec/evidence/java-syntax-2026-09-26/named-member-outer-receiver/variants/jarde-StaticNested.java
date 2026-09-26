// jarde: presentation of `OuterReceiverCases$StaticNested` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class OuterReceiverCases$StaticNested extends java.lang.Object {
    OuterReceiverCases$StaticNested() {
        // @method <init>()V
        // @declaration a constructor of `OuterReceiverCases$StaticNested`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    int read(OuterReceiverCases other) {
        // @method read(LOuterReceiverCases;)I
        // @declaration an instance method of `OuterReceiverCases$StaticNested`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return OuterReceiverCases.access$000(other);
    }
}
