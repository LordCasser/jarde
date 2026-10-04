// jarde: presentation of `ParameterizedSupertypeReturn$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class ParameterizedSupertypeReturn$1 extends Base {
    final java.lang.String val$captured;

    ParameterizedSupertypeReturn$1(java.lang.String arg1) {
        // @method <init>(Ljava/lang/String;)V
        // @declaration a constructor of `ParameterizedSupertypeReturn$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.val$captured = arg1;
        super();
        return;
    }

    public java.lang.String render() {
        // @method render()Ljava/lang/String;
        // @declaration an instance method of `ParameterizedSupertypeReturn$1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append("r:").append(this.val$captured).toString();
    }
}
