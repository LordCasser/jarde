// jarde: presentation of `StringSwitchAudit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class StringSwitchAudit extends java.lang.Object {
    static int calls;

    public StringSwitchAudit() {
        // @method <init>()V
        // @declaration a constructor of `StringSwitchAudit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String read(java.lang.String arg0) {
        // @method read(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `StringSwitchAudit`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        StringSwitchAudit.calls = StringSwitchAudit.calls + 1;
        return arg0;
    }

    public static int choose(java.lang.String arg0) {
        // @method choose(Ljava/lang/String;)I
        // @declaration a static method of `StringSwitchAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        switch (read(arg0)) {
            case "frewhyh":
                return 1;
            case "phgafkp":
                return 2;
            case "test":
            case "test2":
                return 3;
            case "other":
                return 4;
            default:
                return 0;
        }
    }
}
