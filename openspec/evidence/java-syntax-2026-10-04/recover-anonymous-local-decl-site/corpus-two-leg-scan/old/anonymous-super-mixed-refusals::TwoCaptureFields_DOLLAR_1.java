// jarde: presentation of `TwoCaptureFields$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class TwoCaptureFields$1 extends Base {
    final java.lang.String val$first;

    final java.lang.String val$second;

    TwoCaptureFields$1(java.lang.String arg1, int arg2, java.lang.String arg3, java.lang.String arg4) {
        // @method <init>(Ljava/lang/String;ILjava/lang/String;Ljava/lang/String;)V
        // @declaration a constructor of `TwoCaptureFields$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.val$first = arg3;
        this.val$second = arg4;
        super(arg1, arg2);
        return;
    }

    java.lang.String render() {
        // @method render()Ljava/lang/String;
        // @declaration an instance method of `TwoCaptureFields$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        TwoCaptureFields.event(this.val$first);
        TwoCaptureFields.event(this.val$second);
        return super.render();
    }
}
