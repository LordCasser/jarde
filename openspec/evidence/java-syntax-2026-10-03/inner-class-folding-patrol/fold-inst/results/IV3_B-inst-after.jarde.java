// jarde: presentation of `IV3$B` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class IV3$B extends java.lang.Object {
    final IV3 this$0;

    IV3$B(IV3 arg1) {
        // @method <init>(LIV3;)V
        // @declaration a constructor of `IV3$B`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.this$0 = arg1;
        return;
    }

    int b() {
        // @method b()I
        // @declaration an instance method of `IV3$B`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return IV3.access$000(this.this$0) + 1;
    }
}
