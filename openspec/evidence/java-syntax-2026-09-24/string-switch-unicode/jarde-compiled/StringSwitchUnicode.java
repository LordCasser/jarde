// jarde: presentation of `StringSwitchUnicode` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class StringSwitchUnicode extends java.lang.Object {
    private static int calls;

    public StringSwitchUnicode() {
        // @method <init>()V
        // @declaration a constructor of `StringSwitchUnicode`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static java.lang.String selector(java.lang.String arg0) {
        // @method selector(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `StringSwitchUnicode`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        StringSwitchUnicode.calls = StringSwitchUnicode.calls + 1;
        return arg0;
    }

    public static java.lang.String choose(java.lang.String arg0) {
        // @method choose(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `StringSwitchUnicode`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        java.lang.String local1 = selector(arg0);
        local2 = -1;
        switch (local1.hashCode()) {
            case 0:
                if (local1.equals((java.lang.Object) "")) {
                    local2 = 0;
                }
                break;
            case 38634:
                if (local1.equals((java.lang.Object) "雪")) {
                    local2 = 1;
                }
                break;
            case 1770582:
                if (local1.equals((java.lang.Object) "𐐷")) {
                    local2 = 2;
                }
                break;
        }
        switch (local2) {
            case 0:
                return "empty";
            case 1:
                return "bmp";
            case 2:
                return "supplementary";
            default:
                return "default";
        }
    }

    public static void main(java.lang.String[] arg0) {
        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `StringSwitchUnicode`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // BCI 29: the resource's own initialisation is not one statement of this block whose value lands in a slot: writing it in the header would move or drop an effect
        // @bytecode 110 67
        // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [110, 67]
    }

    private static void run(java.lang.String arg0, java.lang.String arg1) {
        // @method run(Ljava/lang/String;Ljava/lang/String;)V
        // @declaration a static method of `StringSwitchUnicode`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        StringSwitchUnicode.calls = 0;
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(arg0).append(":").append((java.lang.String) choose(arg1)).append(":calls=").append(StringSwitchUnicode.calls).toString());
        return;
    }
}
