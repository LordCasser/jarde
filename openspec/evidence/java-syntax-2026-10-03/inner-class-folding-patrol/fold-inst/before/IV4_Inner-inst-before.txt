// jarde: presentation of `IV4$Inner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class IV4$Inner extends java.lang.Object {
    final IV4 this$0;

    IV4$Inner(IV4 arg1) {
        // @method <init>(LIV4;)V
        // @declaration a constructor of `IV4$Inner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.this$0 = arg1;
        return;
    }

    int total() {
        // @method total()I
        // @declaration an instance method of `IV4$Inner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return IV4.access$000(this.this$0);
    }
}
