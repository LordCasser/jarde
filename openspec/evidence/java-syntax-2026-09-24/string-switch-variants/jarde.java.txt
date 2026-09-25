// jarde: presentation of `StringSwitchMiddleDefault` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class StringSwitchMiddleDefault extends java.lang.Object {
    static int calls;

    static int score;

    public StringSwitchMiddleDefault() {
        // @method <init>()V
        // @declaration a constructor of `StringSwitchMiddleDefault`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String read(java.lang.String arg0) {
        // @method read(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `StringSwitchMiddleDefault`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        StringSwitchMiddleDefault.calls = StringSwitchMiddleDefault.calls + 1;
        if ("!".equals((java.lang.Object) arg0)) {
            throw new java.lang.IllegalStateException("selector");
        } else {
            return arg0;
        }
    }

    static void add(int arg0) {
        // @method add(I)V
        // @declaration a static method of `StringSwitchMiddleDefault`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        StringSwitchMiddleDefault.score = StringSwitchMiddleDefault.score * 100 + arg0;
        return;
    }

    public static int choose(java.lang.String arg0) {
        // @method choose(Ljava/lang/String;)I
        // @declaration a static method of `StringSwitchMiddleDefault`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        StringSwitchMiddleDefault.score = 0;
        java.lang.String local1 = read(arg0);
        local2 = -1;
        switch (local1.hashCode()) {
            case -980110702:
                if (local1.equals((java.lang.Object) "prefix")) {
                    local2 = 5;
                }
                break;
            case -891422895:
                if (local1.equals((java.lang.Object) "suffix")) {
                    local2 = 6;
                }
                break;
            case 0:
                if (local1.equals((java.lang.Object) "")) {
                    local2 = 3;
                }
                break;
            case 2112:
                if (local1.equals((java.lang.Object) "BB")) {
                    local2 = 1;
                } else {
                    if (local1.equals((java.lang.Object) "Aa")) {
                        local2 = 0;
                    }
                }
                break;
            case 38634:
                if (local1.equals((java.lang.Object) "雪")) {
                    local2 = 4;
                }
                break;
        }
        switch (local2) {
            case 0:
            case 1:
                add(1);
                break;
            case 2:
            default:
                add(2);
            case 3:
                add(3);
                break;
            case 4:
                add(4);
                break;
            case 5:
                add(5);
            case 6:
                add(6);
                break;
        }
        return StringSwitchMiddleDefault.score;
    }

    public static int once(java.lang.String arg0) {
        // @method once(Ljava/lang/String;)I
        // @declaration a static method of `StringSwitchMiddleDefault`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return choose(arg0);
    }
}
