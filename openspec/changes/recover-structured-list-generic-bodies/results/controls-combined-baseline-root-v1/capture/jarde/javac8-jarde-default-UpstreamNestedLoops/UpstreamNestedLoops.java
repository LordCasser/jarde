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
    public void test(java.util.List l1, java.util.List l2) {
        // @method test(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String s1;
        java.util.Iterator local5;
        for (java.lang.Object iteratorElement17 : l1) {
            s1 = (java.lang.String) iteratorElement17;
            for (java.lang.Object iteratorElement47 : l2) {
                java.lang.String s2 = (java.lang.String) iteratorElement47;
                if (s1.equals((java.lang.Object) s2)) {
                    if (s1.length() == 5) {
                        l2.add((java.lang.Object) s1);
                    } else {
                        l1.remove((java.lang.Object) s2);
                    }
                }
            }
        }
        if (l2.size() > 0) {
            l1.clear();
        }
        return;
    }

    // jarde: generic Signature projection refused for `renamed(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void renamed(java.util.List chosen, java.util.List offered) {
        // @method renamed(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String candidate;
        java.util.Iterator local5;
        for (java.lang.Object iteratorElement17 : chosen) {
            candidate = (java.lang.String) iteratorElement17;
            for (java.lang.Object iteratorElement47 : offered) {
                java.lang.String option = (java.lang.String) iteratorElement47;
                if (candidate.equals((java.lang.Object) option)) {
                    if (candidate.length() == 5) {
                        offered.add((java.lang.Object) candidate);
                    } else {
                        chosen.remove((java.lang.Object) option);
                    }
                }
            }
        }
        if (offered.size() > 0) {
            chosen.clear();
        }
        return;
    }

    // jarde: generic Signature projection refused for `numbers(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void numbers(java.util.List selected, java.util.List available) {
        // @method numbers(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Integer value;
        java.util.Iterator local5;
        for (java.lang.Object iteratorElement17 : selected) {
            value = (java.lang.Integer) iteratorElement17;
            for (java.lang.Object iteratorElement47 : available) {
                java.lang.Integer option = (java.lang.Integer) iteratorElement47;
                if (value.equals((java.lang.Object) option)) {
                    if (value.intValue() == 5) {
                        available.add((java.lang.Object) value);
                    } else {
                        selected.remove((java.lang.Object) option);
                    }
                }
            }
        }
        if (available.size() > 0) {
            selected.clear();
        }
        return;
    }

    // jarde: generic Signature projection refused for `castBoundary(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void castBoundary(java.util.List source, java.util.List destination) {
        // @method castBoundary(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator local3;
        local3 = source.iterator();
        java.lang.String checkedValue;
        java.util.Iterator local6;
        while (local3.hasNext()) {
            java.lang.Object rawValue = local3.next();
            checkedValue = (java.lang.String) rawValue;
            local6 = destination.iterator();
            while (local6.hasNext()) {
                java.lang.Object rawProbe = local6.next();
                java.lang.String checkedProbe = (java.lang.String) rawProbe;
                if (checkedValue.equals((java.lang.Object) checkedProbe)) {
                    destination.add((java.lang.Object) checkedValue);
                    cf11.UpstreamNestedLoops.effects = cf11.UpstreamNestedLoops.effects + 1;
                }
            }
        }
        return;
    }

    public void rawLists(java.util.List left, java.util.List right) {
        // @method rawLists(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator local3;
        local3 = left.iterator();
        java.lang.Object value;
        java.util.Iterator local5;
        while (local3.hasNext()) {
            value = local3.next();
            local5 = right.iterator();
            while (local5.hasNext()) {
                java.lang.Object option = local5.next();
                if (value.equals(option)) {
                    right.add(value);
                    left.remove(option);
                    left.remove(0);
                    cf11.UpstreamNestedLoops.effects = cf11.UpstreamNestedLoops.effects + 1;
                }
            }
        }
        return;
    }

    // jarde: generic Signature projection refused for `rawAliasEscape(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void rawAliasEscape(java.util.List target, java.util.List source) {
        // @method rawAliasEscape(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.util.List escaped;
        java.util.Iterator local4;
        java.lang.Integer value;
        escaped = target;
        local4 = source.iterator();
        while (local4.hasNext()) {
            value = (java.lang.Integer) local4.next();
            escaped.add((java.lang.Object) value);
        }
        local4 = target.iterator();
        while (local4.hasNext()) {
            value = (java.lang.String) local4.next();
            cf11.UpstreamNestedLoops.effects = cf11.UpstreamNestedLoops.effects + 1;
        }
        return;
    }

    // jarde: generic Signature projection refused for `reassigned(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void reassigned(java.util.List input, java.util.List replacement) {
        // @method reassigned(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        input = replacement;
        for (java.lang.Object iteratorElement19 : input) {
            java.lang.String value = (java.lang.String) iteratorElement19;
            replacement.add((java.lang.Object) value);
            cf11.UpstreamNestedLoops.effects = cf11.UpstreamNestedLoops.effects + 1;
        }
        return;
    }

    // jarde: generic Signature projection refused for `wildcardNull(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void wildcardNull(java.util.List values, java.util.List sink) {
        // @method wildcardNull(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        for (java.lang.Object iteratorElement17 : values) {
            java.lang.Number value = (java.lang.Number) iteratorElement17;
            values.add((java.lang.Object) null);
            sink.add((java.lang.Object) value);
            cf11.UpstreamNestedLoops.effects = cf11.UpstreamNestedLoops.effects + 1;
        }
        return;
    }

    // jarde: generic Signature projection refused for `handled(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): method body or no-body declaration has no complete source proof
    public void handled(java.util.List input, java.util.List output) {
        // jarde: not recovered: the recovery run for `handled(Ljava/util/List;Ljava/util/List;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method handled(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 7 16 47 50 59
        // local 3 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
        jarde_refused_body();
    }

    // jarde: generic Signature projection refused for `indexOverload(Ljava/util/List;Ljava/util/List;Ljava/lang/Integer;I)V`: unsupported (ordinary_generic_source_unproved): return source is not a proven parameter value or selected member creation
    public void indexOverload(java.util.List values, java.util.List witness, java.lang.Integer key, int position) {
        // @method indexOverload(Ljava/util/List;Ljava/util/List;Ljava/lang/Integer;I)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        values.remove((java.lang.Object) key);
        values.remove(position);
        witness.add((java.lang.Object) key);
        cf11.UpstreamNestedLoops.effects = cf11.UpstreamNestedLoops.effects + 1;
        return;
    }
}
