// jarde: presentation of `AnonymousCaptureCases$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class AnonymousCaptureCases$1 extends AnonymousCaptureCases$Base {
    final java.lang.String val$captured;

    AnonymousCaptureCases$1(long seed, java.lang.String arg3) {
        // @method <init>(JLjava/lang/String;)V
        // @declaration a constructor of `AnonymousCaptureCases$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.val$captured = arg3;
        super(seed);
        return;
    }

    public java.lang.String render() {
        // @method render()Ljava/lang/String;
        // @declaration an instance method of `AnonymousCaptureCases$1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        AnonymousCaptureCases.access$208();
        AnonymousCaptureCases.access$100("render");
        return new java.lang.StringBuilder().append(this.seed()).append(":").append(this.val$captured).toString();
    }
}
