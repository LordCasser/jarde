// jarde: presentation of `NamedMemberFamilyStage1$Member` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class NamedMemberFamilyStage1$Member extends java.lang.Object {
    final NamedMemberFamilyStage1 this$0;

    NamedMemberFamilyStage1$Member(NamedMemberFamilyStage1 this$0) {
        // @method <init>(LNamedMemberFamilyStage1;)V
        // @declaration a constructor of `NamedMemberFamilyStage1$Member`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.this$0 = this$0;
        super();
        return;
    }

    int read(NamedMemberFamilyStage1 other) {
        // @method read(LNamedMemberFamilyStage1;)I
        // @declaration an instance method of `NamedMemberFamilyStage1$Member`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return other.state * 100 + NamedMemberFamilyStage1.access$000(this.this$0);
    }
}
