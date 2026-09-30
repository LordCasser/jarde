// jarde: presentation of `Tf1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Tf1 extends java.lang.Object {
    public Tf1() {
        // @method <init>()V
        // @declaration a constructor of `Tf1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    java.lang.String test(Context arg1, java.lang.Object arg2) {
        // @method test(LContext;Ljava/lang/Object;)Ljava/lang/String;
        // @declaration an instance method of `Tf1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        Cursor local3;
        java.lang.String local6;
        local3 = null;
        try {
            java.lang.String[] local4 = new java.lang.String[]{"name"};
            local3 = arg1.query(arg2, local4);
            int local5 = local3.getColumnIndexOrThrow("name");
            local3.moveToFirst();
            local6 = local3.getString(local5);
            return local6;
        } finally {
            if (local3 != null) {
                local3.close();
            }
        }
    }
}
