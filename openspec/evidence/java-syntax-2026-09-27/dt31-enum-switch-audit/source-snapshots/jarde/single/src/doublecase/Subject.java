// jarde: presentation of `doublecase/Subject` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package doublecase;

public final class Subject extends java.lang.Object {
    public Subject() {
        // @method <init>()V
        // @declaration a constructor of `doublecase.Subject`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int select(doublecase.Count arg0, doublecase.Animal arg1) {
        // @method select(Ldoublecase/Count;Ldoublecase/Animal;)I
        // @declaration a static method of `doublecase.Subject`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        local2 = 0;
        switch (doublecase.Subject$1.$SwitchMap$doublecase$Count[arg0.ordinal()]) {
            case 1:
                local2 = 1;
                break;
            case 2:
                local2 = 2;
                break;
        }
        switch (doublecase.Subject$1.$SwitchMap$doublecase$Animal[arg1.ordinal()]) {
            case 1:
                local2 = local2 + 10;
                break;
            case 2:
                local2 = local2 + 20;
                break;
        }
        return local2;
    }
}
