// jarde: presentation of `HiddenTypeUse` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class HiddenTypeUse extends java.lang.Object {
    java.lang.@HiddenMark(value = "field") String field;

    HiddenTypeUse() {
        // @method <init>()V
        // @declaration a constructor of `HiddenTypeUse`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    java.lang.@HiddenMark(value = "return") String value() {
        // @method value()Ljava/lang/String;
        // @declaration an instance method of `HiddenTypeUse`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return "value";
    }

    java.lang.String echo(java.lang.@HiddenMark(value = "parameter") String arg1) {
        // @method echo(Ljava/lang/String;)Ljava/lang/String;
        // @declaration an instance method of `HiddenTypeUse`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return arg1;
    }
}
