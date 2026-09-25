// jarde: presentation of `demo/PlainRunner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package demo;

public class PlainRunner extends java.lang.Object {
    public PlainRunner() {
        // @method <init>()V
        // @declaration a constructor of `demo.PlainRunner`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `demo.PlainRunner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        demo.Plain[] local1;
        int local2;
        int local3;
        local1 = demo.Plain.values();
        local2 = local1.length;
        local3 = 0;
        while (local3 < local2) {
            Object value = local1[local3];
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
            // @bytecode 42 44
            // the saved producer at BCI 44 has no bounded final expression consumer
            // @bytecode 22 25 26 29 31 34 39 42 44 47
            // the saved producer at BCI 47 has no bounded final expression consumer
            // @bytecode 22 25 26 29 31 34 39 42 44 47 52
            // the saved producer at BCI 52 has 3 consumers, so one local binding cannot prove its execution count
            // @bytecode 76 19 73 70 52 47 39 34 26 22 25 31 44 29 42
            // the value at BCI 76 was produced by a saved declaration this run could not commit
            local3 = local3 + 1;
        }
        return;
    }
}
