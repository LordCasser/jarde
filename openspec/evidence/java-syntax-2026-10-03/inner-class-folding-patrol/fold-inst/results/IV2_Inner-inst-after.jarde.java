// jarde: presentation of `IV2$Inner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class IV2$Inner extends java.lang.Object {
    final IV2 this$0;

    IV2$Inner(IV2 arg1) {
        // @method <init>(LIV2;)V
        // @declaration a constructor of `IV2$Inner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.this$0 = arg1;
        return;
    }

    void bump() {
        // @method bump()V
        // @declaration an instance method of `IV2$Inner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        IV2.access$002(this.this$0, IV2.access$000(this.this$0) + 1);
        return;
    }
}
