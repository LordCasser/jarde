// jarde: presentation of `TypeUseSubject` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class TypeUseSubject extends java.lang.Object {
    java.lang.String field;

    TypeUseSubject() {
        // @method <init>()V
        // @declaration a constructor of `TypeUseSubject`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    java.lang.String value() {
        // @method value()Ljava/lang/String;
        // @declaration an instance method of `TypeUseSubject`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return "value";
    }

    java.lang.String echo(java.lang.String arg1) {
        // @method echo(Ljava/lang/String;)Ljava/lang/String;
        // @declaration an instance method of `TypeUseSubject`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return arg1;
    }
}
