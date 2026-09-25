// jarde: presentation of `EnumSwitchSubject` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class EnumSwitchSubject extends java.lang.Object {
    private static int trace;

    public EnumSwitchSubject() {
        // @method <init>()V
        // @declaration a constructor of `EnumSwitchSubject`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int mark(int arg0) {
        // @method mark(I)I
        // @declaration a static method of `EnumSwitchSubject`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        EnumSwitchSubject.trace = EnumSwitchSubject.trace * 10 + arg0;
        return arg0;
    }

    public static int trace() {
        // @method trace()I
        // @declaration a static method of `EnumSwitchSubject`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return EnumSwitchSubject.trace;
    }

    public static void reset() {
        // @method reset()V
        // @declaration a static method of `EnumSwitchSubject`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        EnumSwitchSubject.trace = 0;
        return;
    }

    public static int choose(Hue arg0) {
        // @method choose(LHue;)I
        // @declaration a static method of `EnumSwitchSubject`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        switch (EnumSwitchSubject$1.$SwitchMap$Hue[arg0.ordinal()]) {
            case 1:
                return mark(1);
            case 2:
                return mark(2);
            default:
                return mark(3);
        }
    }
}
