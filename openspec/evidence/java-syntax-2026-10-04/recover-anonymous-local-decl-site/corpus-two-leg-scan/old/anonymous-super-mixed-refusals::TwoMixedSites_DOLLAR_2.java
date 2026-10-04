// jarde: presentation of `TwoMixedSites$2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class TwoMixedSites$2 extends Base {
    final java.lang.String val$captured;

    TwoMixedSites$2(java.lang.String arg1, int arg2, java.lang.String arg3) {
        // @method <init>(Ljava/lang/String;ILjava/lang/String;)V
        // @declaration a constructor of `TwoMixedSites$2`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.val$captured = arg3;
        super(arg1, arg2);
        return;
    }

    java.lang.String render() {
        // @method render()Ljava/lang/String;
        // @declaration an instance method of `TwoMixedSites$2`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        TwoMixedSites.event(this.val$captured);
        return super.render();
    }
}
