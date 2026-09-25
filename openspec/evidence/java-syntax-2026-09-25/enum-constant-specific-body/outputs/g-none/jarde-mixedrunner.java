// jarde: presentation of `demo/MixedRunner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package demo;

public class MixedRunner extends java.lang.Object {
    public MixedRunner() {
        // @method <init>()V
        // @declaration a constructor of `demo.MixedRunner`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `demo.MixedRunner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        demo.Mixed[] local1;
        int local2;
        int local3;
        local1 = demo.Mixed.values();
        local2 = local1.length;
        local3 = 0;
        Object local4;
        while (local3 < local2) {
            local4 = local1[local3];
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
            // the saved producer at BCI 52 has no bounded final expression consumer
            // @bytecode 55 57
            // the saved producer at BCI 57 has no bounded final expression consumer
            // @bytecode 22 25 26 29 31 34 39 42 44 47 52 55 57 60
            // the saved producer at BCI 60 has no bounded final expression consumer
            // @bytecode 22 25 26 29 31 34 39 42 44 47 52 55 57 60 65
            // the saved producer at BCI 65 has 3 consumers, so one local binding cannot prove its execution count
            // @bytecode 121 19 118 115 104 99 88 83 65 60 52 47 39 34 26 22 25 31 44 57 96 93 112 109 29 42 55 91 107
            // the value at BCI 121 was produced by a saved declaration this run could not commit
            local3 = local3 + 1;
        }
        return;
    }
}
