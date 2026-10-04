// jarde: presentation of `ParameterizedAlsoConsumed$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class ParameterizedAlsoConsumed$1 extends Base {
    final java.lang.String val$captured;

    ParameterizedAlsoConsumed$1(java.lang.String arg1) {
        // @method <init>(Ljava/lang/String;)V
        // @declaration a constructor of `ParameterizedAlsoConsumed$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.val$captured = arg1;
        super();
        return;
    }

    void observe() {
        // @method observe()V
        // @declaration an instance method of `ParameterizedAlsoConsumed$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        ParameterizedAlsoConsumed.observed = this.val$captured;
        return;
    }
}
