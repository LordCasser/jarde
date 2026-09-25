// jarde: presentation of `demo/Runner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package demo;

public class Runner extends java.lang.Object {
    public Runner() {
        // @method <init>()V
        // @declaration a constructor of `demo.Runner`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `demo.Runner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        demo.Op[] local1;
        int local2;
        int local3;
        local1 = demo.Op.values();
        local2 = local1.length;
        local3 = 0;
        Object op;
        while (local3 < local2) {
            op = local1[local3];
            // @bytecode 19
            // the saved producer at BCI 19 has 3 consumers, so one local binding cannot prove its execution count
            // @bytecode 22 25 26
            // the saved producer at BCI 26 has no bounded final expression consumer
            // @bytecode 29 31
            // the saved producer at BCI 31 has no bounded final expression consumer
            // @bytecode 22 25 26 29 31 34
            // the saved producer at BCI 34 has no bounded final expression consumer
            // @bytecode 22 25 26 29 31 34 39
            // the saved producer at BCI 39 has no bounded final expression consumer
            // @bytecode 42 47
            // the saved producer at BCI 47 has no bounded final expression consumer
            // @bytecode 22 25 26 29 31 34 39 42 47 50
            // the saved producer at BCI 50 has no bounded final expression consumer
            // @bytecode 22 25 26 29 31 34 39 42 47 50 55
            // the saved producer at BCI 55 has 3 consumers, so one local binding cannot prove its execution count
            // @bytecode 111 19 108 105 94 89 78 73 55 50 39 34 26 22 25 31 47 86 83 102 99 29 42 81 97
            // the value at BCI 111 was produced by a saved declaration this run could not commit
            local3 = local3 + 1;
        }
        return;
    }
}
