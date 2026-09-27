// jarde: presentation of `cf08/NotIndexedLoop` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf08;

public final class NotIndexedLoop extends java.lang.Object {
    public NotIndexedLoop() {
        // @method <init>()V
        // @declaration a constructor of `cf08.NotIndexedLoop`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.io.File test(java.io.File[] arg1) {
        // @method test([Ljava/io/File;)Ljava/io/File;
        // @declaration an instance method of `cf08.NotIndexedLoop`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.io.File local2;
        if (arg1 != null) {
            int local3 = arg1.length;
            if (local3 == 0) {
                local2 = null;
            } else {
                int local4;
                local4 = 0;
                while (true) {
                    if (local4 >= local3) {
                        local2 = new java.io.File("h");
                        break;
                    } else {
                        local2 = arg1[local4];
                        if (local2.getName().equals((java.lang.Object) "f")) {
                            break;
                        } else {
                            local4 = local4 + 1;
                        }
                    }
                }
            }
        } else {
            local2 = null;
        }
        if (local2 != null) {
            local2.deleteOnExit();
        }
        return local2;
    }
}
