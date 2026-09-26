// jarde: presentation of `AnonymousSuperArgs$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class AnonymousSuperArgs$1 extends Base {
    final java.lang.String val$captured;

    AnonymousSuperArgs$1(java.lang.String arg1, int arg2, java.lang.String arg3) {
        // @method <init>(Ljava/lang/String;ILjava/lang/String;)V
        // @declaration a constructor of `AnonymousSuperArgs$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.val$captured = arg3;
        super(arg1, arg2);
        return;
    }

    java.lang.String render() {
        // @method render()Ljava/lang/String;
        // @declaration an instance method of `AnonymousSuperArgs$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        AnonymousSuperArgs.event((java.lang.String) new java.lang.StringBuilder().append("body:").append(this.val$captured).toString());
        return new java.lang.StringBuilder().append((java.lang.String) super.render()).append(":").append(this.val$captured).toString();
    }
}
