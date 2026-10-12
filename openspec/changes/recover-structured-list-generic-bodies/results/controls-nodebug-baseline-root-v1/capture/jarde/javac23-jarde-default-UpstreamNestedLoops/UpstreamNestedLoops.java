// jarde: presentation of `cf11/UpstreamNestedLoops` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class UpstreamNestedLoops extends java.lang.Object {
    public static int effects;

    public UpstreamNestedLoops() {
        // @method <init>()V
        // @declaration a constructor of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `test(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void test(java.util.List arg1, java.util.List arg2) {
        // @method test(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local4;
        java.util.Iterator local5;
        for (java.lang.Object iteratorElement17 : arg1) {
            local4 = (java.lang.String) iteratorElement17;
            for (java.lang.Object iteratorElement47 : arg2) {
                java.lang.String local6 = (java.lang.String) iteratorElement47;
                if (local4.equals((java.lang.Object) local6)) {
                    if (local4.length() == 5) {
                        arg2.add((java.lang.Object) local4);
                    } else {
                        arg1.remove((java.lang.Object) local6);
                    }
                }
            }
        }
        if (arg2.size() > 0) {
            arg1.clear();
        }
        return;
    }

    // jarde: generic Signature projection refused for `renamed(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void renamed(java.util.List arg1, java.util.List arg2) {
        // @method renamed(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local4;
        java.util.Iterator local5;
        for (java.lang.Object iteratorElement17 : arg1) {
            local4 = (java.lang.String) iteratorElement17;
            for (java.lang.Object iteratorElement47 : arg2) {
                java.lang.String local6 = (java.lang.String) iteratorElement47;
                if (local4.equals((java.lang.Object) local6)) {
                    if (local4.length() == 5) {
                        arg2.add((java.lang.Object) local4);
                    } else {
                        arg1.remove((java.lang.Object) local6);
                    }
                }
            }
        }
        if (arg2.size() > 0) {
            arg1.clear();
        }
        return;
    }

    // jarde: generic Signature projection refused for `numbers(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void numbers(java.util.List arg1, java.util.List arg2) {
        // @method numbers(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Integer local4;
        java.util.Iterator local5;
        for (java.lang.Object iteratorElement17 : arg1) {
            local4 = (java.lang.Integer) iteratorElement17;
            for (java.lang.Object iteratorElement47 : arg2) {
                java.lang.Integer local6 = (java.lang.Integer) iteratorElement47;
                if (local4.equals((java.lang.Object) local6)) {
                    if (local4.intValue() == 5) {
                        arg2.add((java.lang.Object) local4);
                    } else {
                        arg1.remove((java.lang.Object) local6);
                    }
                }
            }
        }
        if (arg2.size() > 0) {
            arg1.clear();
        }
        return;
    }

    // jarde: generic Signature projection refused for `castBoundary(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void castBoundary(java.util.List arg1, java.util.List arg2) {
        // @method castBoundary(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator local3;
        local3 = arg1.iterator();
        java.lang.String local5;
        java.util.Iterator local6;
        while (local3.hasNext()) {
            java.lang.Object local4 = local3.next();
            local5 = (java.lang.String) local4;
            local6 = arg2.iterator();
            while (local6.hasNext()) {
                java.lang.Object local7 = local6.next();
                java.lang.String local8 = (java.lang.String) local7;
                if (local5.equals((java.lang.Object) local8)) {
                    arg2.add((java.lang.Object) local5);
                    cf11.UpstreamNestedLoops.effects = cf11.UpstreamNestedLoops.effects + 1;
                }
            }
        }
        return;
    }

    public void rawLists(java.util.List arg1, java.util.List arg2) {
        // @method rawLists(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator local3;
        local3 = arg1.iterator();
        java.lang.Object local4;
        java.util.Iterator local5;
        while (local3.hasNext()) {
            local4 = local3.next();
            local5 = arg2.iterator();
            while (local5.hasNext()) {
                java.lang.Object local6 = local5.next();
                if (local4.equals(local6)) {
                    arg2.add(local4);
                    arg1.remove(local6);
                    arg1.remove(0);
                    cf11.UpstreamNestedLoops.effects = cf11.UpstreamNestedLoops.effects + 1;
                }
            }
        }
        return;
    }

    // jarde: generic Signature projection refused for `rawAliasEscape(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void rawAliasEscape(java.util.List arg1, java.util.List arg2) {
        // @method rawAliasEscape(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.util.List local3;
        java.util.Iterator local4;
        local3 = arg1;
        local4 = arg2.iterator();
        while (local4.hasNext()) {
            java.lang.Integer local5 = (java.lang.Integer) local4.next();
            local3.add((java.lang.Object) local5);
        }
        local4 = arg1.iterator();
        while (local4.hasNext()) {
            java.lang.String local5_2 = (java.lang.String) local4.next();
            cf11.UpstreamNestedLoops.effects = cf11.UpstreamNestedLoops.effects + 1;
        }
        return;
    }

    // jarde: generic Signature projection refused for `reassigned(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void reassigned(java.util.List arg1, java.util.List arg2) {
        // @method reassigned(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        arg1 = arg2;
        for (java.lang.Object iteratorElement19 : arg1) {
            java.lang.String local4 = (java.lang.String) iteratorElement19;
            arg2.add((java.lang.Object) local4);
            cf11.UpstreamNestedLoops.effects = cf11.UpstreamNestedLoops.effects + 1;
        }
        return;
    }

    // jarde: generic Signature projection refused for `wildcardNull(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void wildcardNull(java.util.List arg1, java.util.List arg2) {
        // @method wildcardNull(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        for (java.lang.Object iteratorElement17 : arg1) {
            java.lang.Number local4 = (java.lang.Number) iteratorElement17;
            arg1.add((java.lang.Object) null);
            arg2.add((java.lang.Object) local4);
            cf11.UpstreamNestedLoops.effects = cf11.UpstreamNestedLoops.effects + 1;
        }
        return;
    }

    // jarde: generic Signature projection refused for `handled(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): method body or no-body declaration has no complete source proof
    public void handled(java.util.List arg1, java.util.List arg2) {
        // jarde: not recovered: the recovery run for `handled(Ljava/util/List;Ljava/util/List;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method handled(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 7 16 47 50 59
        // local 3 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
        jarde_refused_body();
    }

    // jarde: generic Signature projection refused for `indexOverload(Ljava/util/List;Ljava/util/List;Ljava/lang/Integer;I)V`: unsupported (ordinary_generic_source_unproved): return source is not a proven parameter value or selected member creation
    public void indexOverload(java.util.List arg1, java.util.List arg2, java.lang.Integer arg3, int arg4) {
        // @method indexOverload(Ljava/util/List;Ljava/util/List;Ljava/lang/Integer;I)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        arg1.remove((java.lang.Object) arg3);
        arg1.remove(arg4);
        arg2.add((java.lang.Object) arg3);
        cf11.UpstreamNestedLoops.effects = cf11.UpstreamNestedLoops.effects + 1;
        return;
    }
}
