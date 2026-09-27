// jarde: presentation of `CatchShapes` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class CatchShapes extends java.lang.Object {
    public CatchShapes() {
        // @method <init>()V
        // @declaration a constructor of `CatchShapes`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String ordinary(int kind) {
        // @method ordinary(I)Ljava/lang/String;
        // @declaration a static method of `CatchShapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (kind == 1) {
                throw new java.lang.IllegalArgumentException("arg");
            } else if (kind == 2) {
                throw new java.lang.IllegalStateException("state");
    } else {
                return "ok";
    }
        } catch (java.lang.IllegalArgumentException error) {
            return "argument:" + error.getMessage();
        } catch (java.lang.IllegalStateException error) {
            return new java.lang.StringBuilder().append("state:").append((java.lang.String) error.getMessage()).toString();
        }
    }

    public static java.lang.String multi(boolean state) {
        // @method multi(Z)Ljava/lang/String;
        // @declaration a static method of `CatchShapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (state) {
                throw new java.lang.IllegalStateException("multi");
            } else {
                throw new java.lang.IllegalArgumentException("multi");
            }
        } catch (java.lang.IllegalArgumentException | java.lang.IllegalStateException error) {
            return error.getClass().getSimpleName() + ":" + error.getMessage();
        }
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `CatchShapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) ordinary(0));
        java.lang.System.out.println((java.lang.String) ordinary(1));
        java.lang.System.out.println((java.lang.String) ordinary(2));
        java.lang.System.out.println((java.lang.String) multi(false));
        java.lang.System.out.println((java.lang.String) multi(true));
        return;
    }
}
