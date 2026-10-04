// jarde: presentation of `ParameterizedMultiParams$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class ParameterizedMultiParams$1 extends Base {
    final java.lang.String val$captured;

    ParameterizedMultiParams$1(java.lang.String arg1) {
        // @method <init>(Ljava/lang/String;)V
        // @declaration a constructor of `ParameterizedMultiParams$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.val$captured = arg1;
        super();
        return;
    }

    void observe() {
        // @method observe()V
        // @declaration an instance method of `ParameterizedMultiParams$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        ParameterizedMultiParams.observed = this.val$captured;
        return;
    }
}
