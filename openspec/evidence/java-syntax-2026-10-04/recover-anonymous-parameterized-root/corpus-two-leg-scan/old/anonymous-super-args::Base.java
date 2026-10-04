// jarde: presentation of `Base` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class Base extends java.lang.Object {
    private final java.lang.String label;

    private final int value;

    Base(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.label = arg1;
        this.value = arg2;
        AnonymousSuperArgs.event("base:" + arg1 + ":" + arg2);
        return;
    }

    java.lang.String render() {
        // @method render()Ljava/lang/String;
        // @declaration an instance method of `Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append(this.label).append(":").append(this.value).toString();
    }
}
