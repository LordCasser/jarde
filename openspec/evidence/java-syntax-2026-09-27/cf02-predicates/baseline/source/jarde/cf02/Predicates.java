// jarde: presentation of `cf02/Predicates` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf02;

public class Predicates extends java.lang.Object {
    public Predicates() {
        // @method <init>()V
        // @declaration a constructor of `cf02.Predicates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public boolean greater(float arg1, float arg2) {
        // @method greater(FF)Z
        // @declaration an instance method of `cf02.Predicates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 > arg2;
    }

    public boolean mixed(float arg1, double arg2) {
        // @method mixed(FD)Z
        // @declaration an instance method of `cf02.Predicates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return (double) arg1 < arg2;
    }

    public boolean inBounds(int[] arg1, int arg2) {
        // @method inBounds([II)Z
        // @declaration an instance method of `cf02.Predicates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg2 >= 0 && arg2 < arg1.length;
    }

    public boolean named(java.lang.Object arg1) {
        // @method named(Ljava/lang/Object;)Z
        // @declaration an instance method of `cf02.Predicates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        if (arg1 != null) {
            if (!(arg1 instanceof java.lang.String)) {
            }
        }
        return false;
        // @bytecode 28
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [28]
    }
}
