// jarde: presentation of `AnonymousMemberBase$Outer$Base` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class AnonymousMemberBase$Outer$Base extends java.lang.Object {
    final int value;

    final AnonymousMemberBase$Outer this$0;

    AnonymousMemberBase$Outer$Base(AnonymousMemberBase$Outer this$0, int value) {
        // @method <init>(LAnonymousMemberBase$Outer;I)V
        // @declaration a constructor of `AnonymousMemberBase$Outer$Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.this$0 = this$0;
        this.value = value;
        AnonymousMemberBase.access$000("base(" + value + ")");
        return;
    }

    int render() {
        // @method render()I
        // @declaration an instance method of `AnonymousMemberBase$Outer$Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.value;
    }
}
