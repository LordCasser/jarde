// jarde: presentation of `em12/Case` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em12;

public class Case extends em12.Parent {
    public Case() {
        // @method <init>()V
        // @declaration a constructor of `em12.Case`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `em12.Case`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        em12.Case local1 = new em12.Case();
        java.io.PrintStream local2 = java.lang.System.out;
        java.util.Objects.requireNonNull((java.lang.Object) local1);
        local2.println((java.lang.String) local1.new Member().call((em12.Arg) null));
        return;
    }

    class Member extends java.lang.Object {
        Member() {
            super();
            return;
        }

        java.lang.String call(em12.Arg arg1) {
            // @method call(Lem12/Arg;)Ljava/lang/String;
            // @declaration an instance method of `em12.Case$Member`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return em12.Case.super.pick(arg1);
        }
    }
}
