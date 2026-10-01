// jarde: presentation of `CWV` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class CWV extends java.lang.Object {
    public CWV() {
        // @method <init>()V
        // @declaration a constructor of `CWV`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `mapValue(Ljava/util/Map;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static java.lang.String mapValue(java.util.Map arg0) {
        // @method mapValue(Ljava/util/Map;)Ljava/lang/String;
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return (java.lang.String) arg0.get((java.lang.Object) "k");
    }

    // jarde: generic Signature projection refused for `mapSize(Ljava/util/Map;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int mapSize(java.util.Map arg0) {
        // @method mapSize(Ljava/util/Map;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.size();
    }

    // jarde: generic Signature projection refused for `setSize(Ljava/util/Set;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int setSize(java.util.Set arg0) {
        // @method setSize(Ljava/util/Set;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.size();
    }

    // jarde: generic Signature projection refused for `collectionSize(Ljava/util/Collection;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int collectionSize(java.util.Collection arg0) {
        // @method collectionSize(Ljava/util/Collection;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.size();
    }

    // jarde: generic Signature projection refused for `iterableSize(Ljava/lang/Iterable;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int iterableSize(java.lang.Iterable arg0) {
        // @method iterableSize(Ljava/lang/Iterable;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        java.util.Iterator local2;
        local1 = 0;
        local2 = arg0.iterator();
        while (local2.hasNext()) {
            java.lang.String local3 = (java.lang.String) local2.next();
            if (local3 != null) {
                local1 = local1 + 1;
            }
        }
        return local1;
    }

    // jarde: generic Signature projection refused for `queueSize(Ljava/util/Queue;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int queueSize(java.util.Queue arg0) {
        // @method queueSize(Ljava/util/Queue;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.size();
    }

    // jarde: generic Signature projection refused for `sortedSetSize(Ljava/util/SortedSet;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int sortedSetSize(java.util.SortedSet arg0) {
        // @method sortedSetSize(Ljava/util/SortedSet;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.size();
    }

    // jarde: generic Signature projection refused for `sortedMapValue(Ljava/util/SortedMap;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static java.lang.String sortedMapValue(java.util.SortedMap arg0) {
        // @method sortedMapValue(Ljava/util/SortedMap;)Ljava/lang/String;
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return (java.lang.String) arg0.get((java.lang.Object) "k");
    }

    // jarde: generic Signature projection refused for `listValue(Ljava/util/List;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static java.lang.String listValue(java.util.List arg0) {
        // @method listValue(Ljava/util/List;)Ljava/lang/String;
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return (java.lang.String) arg0.get(0);
    }

    // jarde: generic Signature projection refused for `nestedListSize(Ljava/util/List;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int nestedListSize(java.util.List arg0) {
        // @method nestedListSize(Ljava/util/List;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.size() + ((java.util.List) arg0.get(0)).size();
    }

    // jarde: generic Signature projection refused for `dequeSize(Ljava/util/Deque;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int dequeSize(java.util.Deque arg0) {
        // @method dequeSize(Ljava/util/Deque;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.size();
    }

    // jarde: generic Signature projection refused for `dequeToQueue(Ljava/util/Deque;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int dequeToQueue(java.util.Deque arg0) {
        // @method dequeToQueue(Ljava/util/Deque;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return queueSize((java.util.Queue) arg0);
    }

    // jarde: generic Signature projection refused for `sortedSetToSet(Ljava/util/SortedSet;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int sortedSetToSet(java.util.SortedSet arg0) {
        // @method sortedSetToSet(Ljava/util/SortedSet;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return setSize((java.util.Set) arg0);
    }

    // jarde: generic Signature projection refused for `navigableSetToSortedSet(Ljava/util/NavigableSet;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int navigableSetToSortedSet(java.util.NavigableSet arg0) {
        // @method navigableSetToSortedSet(Ljava/util/NavigableSet;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return sortedSetSize((java.util.SortedSet) arg0);
    }

    // jarde: generic Signature projection refused for `navigableMapToSortedMap(Ljava/util/NavigableMap;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static java.lang.String navigableMapToSortedMap(java.util.NavigableMap arg0) {
        // @method navigableMapToSortedMap(Ljava/util/NavigableMap;)Ljava/lang/String;
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return sortedMapValue((java.util.SortedMap) arg0);
    }

    // jarde: generic Signature projection refused for `listToCollection(Ljava/util/List;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int listToCollection(java.util.List arg0) {
        // @method listToCollection(Ljava/util/List;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return collectionSize((java.util.Collection) arg0);
    }

    // jarde: generic Signature projection refused for `setToCollection(Ljava/util/Set;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int setToCollection(java.util.Set arg0) {
        // @method setToCollection(Ljava/util/Set;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return collectionSize((java.util.Collection) arg0);
    }

    // jarde: generic Signature projection refused for `queueToCollection(Ljava/util/Queue;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int queueToCollection(java.util.Queue arg0) {
        // @method queueToCollection(Ljava/util/Queue;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return collectionSize((java.util.Collection) arg0);
    }

    // jarde: generic Signature projection refused for `collectionToIterable(Ljava/util/Collection;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int collectionToIterable(java.util.Collection arg0) {
        // @method collectionToIterable(Ljava/util/Collection;)I
        // @declaration a static method of `CWV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return iterableSize((java.lang.Iterable) arg0);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `CWV`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.HashMap local1 = new java.util.HashMap();
        local1.put((java.lang.Object) "k", (java.lang.Object) "hashmap");
        java.lang.System.out.println("map:" + mapValue((java.util.Map) local1));
        java.util.TreeMap local2 = new java.util.TreeMap();
        local2.put((java.lang.Object) "k", (java.lang.Object) "treemap");
        java.lang.System.out.println("treemap:" + mapValue((java.util.Map) local2));
        java.util.LinkedHashMap local3 = new java.util.LinkedHashMap();
        local3.put((java.lang.Object) "k", (java.lang.Object) "linkedmap");
        java.lang.System.out.println("linkedmap:" + mapValue((java.util.Map) local3));
        java.util.Hashtable local4 = new java.util.Hashtable();
        local4.put((java.lang.Object) "k", (java.lang.Object) "hashtable");
        java.lang.System.out.println("hashtable:" + mapValue((java.util.Map) local4));
        java.util.Properties local5 = new java.util.Properties();
        local5.setProperty("k", "properties");
        java.lang.System.out.println("properties:" + mapSize((java.util.Map) local5));
        java.util.HashSet local6 = new java.util.HashSet();
        local6.add((java.lang.Object) "hashset");
        java.lang.System.out.println("set:" + setSize((java.util.Set) local6));
        java.util.TreeSet local7 = new java.util.TreeSet();
        local7.add((java.lang.Object) "treeset");
        java.lang.System.out.println("treeset:" + setSize((java.util.Set) local7));
        java.util.LinkedHashSet local8 = new java.util.LinkedHashSet();
        local8.add((java.lang.Object) "linkedhashset");
        java.lang.System.out.println("linkedhashset:" + setSize((java.util.Set) local8));
        java.util.ArrayList local9 = new java.util.ArrayList();
        local9.add((java.lang.Object) "coll");
        local9.add((java.lang.Object) "two");
        java.lang.System.out.println("collection:" + collectionSize((java.util.Collection) local9));
        java.lang.System.out.println("list:" + listValue((java.util.List) local9));
        java.lang.System.out.println("iterable:" + iterableSize((java.lang.Iterable) local9));
        java.util.LinkedList local10 = new java.util.LinkedList();
        local10.add((java.lang.Object) "linked");
        java.lang.System.out.println("linkedlist:" + listValue((java.util.List) local10));
        java.util.Vector local11 = new java.util.Vector();
        local11.add((java.lang.Object) "vector");
        java.lang.System.out.println("vector:" + collectionSize((java.util.Collection) local11));
        java.util.Stack local12 = new java.util.Stack();
        local12.push((java.lang.Object) "stack");
        java.lang.System.out.println("stack:" + listValue((java.util.List) local12));
        java.util.ArrayList local13 = new java.util.ArrayList();
        java.util.ArrayList local14 = new java.util.ArrayList();
        local14.add((java.lang.Object) "inner");
        local13.add((java.lang.Object) local14);
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("nestedList:").append((java.lang.String) listValue((java.util.List) local13.get(0))).toString());
        java.util.ArrayList local15 = new java.util.ArrayList();
        local15.add((java.lang.Object) local14);
        java.lang.System.out.println("nested:" + nestedListSize((java.util.List) local15));
        java.util.ArrayDeque local16 = new java.util.ArrayDeque();
        local16.add((java.lang.Object) "deque");
        java.lang.System.out.println("deque:" + dequeSize((java.util.Deque) local16));
        java.lang.System.out.println("dequeCollection:" + collectionSize((java.util.Collection) local16));
        java.util.ArrayList local17 = new java.util.ArrayList();
        local17.add((java.lang.Object) "listinterface");
        local17.add((java.lang.Object) "second");
        java.lang.System.out.println("listToCollection:" + listToCollection((java.util.List) local17));
        java.util.HashSet local18 = new java.util.HashSet();
        local18.add((java.lang.Object) "setinterface");
        java.lang.System.out.println("setToCollection:" + setToCollection((java.util.Set) local18));
        java.util.LinkedList local19 = new java.util.LinkedList();
        local19.add((java.lang.Object) "queueinterface");
        java.lang.System.out.println("queueToCollection:" + queueToCollection((java.util.Queue) local19));
        java.util.ArrayList local20 = new java.util.ArrayList();
        local20.add((java.lang.Object) "collectioninterface");
        local20.add((java.lang.Object) "another");
        java.lang.System.out.println("collectionToIterable:" + collectionToIterable((java.util.Collection) local20));
        java.util.ArrayDeque local21 = new java.util.ArrayDeque();
        local21.add((java.lang.Object) "dequeinterface");
        java.lang.System.out.println("dequeToQueue:" + dequeToQueue((java.util.Deque) local21));
        java.util.TreeSet local22 = new java.util.TreeSet();
        local22.add((java.lang.Object) "sortedsetinterface");
        java.lang.System.out.println("sortedSetToSet:" + sortedSetToSet((java.util.SortedSet) local22));
        java.util.TreeSet local23 = new java.util.TreeSet();
        local23.add((java.lang.Object) "navigablesetinterface");
        java.lang.System.out.println("navigableSetToSortedSet:" + navigableSetToSortedSet((java.util.NavigableSet) local23));
        java.util.TreeMap local24 = new java.util.TreeMap();
        local24.put((java.lang.Object) "k", (java.lang.Object) "navigablemapinterface");
        java.lang.System.out.println("navigableMapToSortedMap:" + navigableMapToSortedMap((java.util.NavigableMap) local24));
        return;
    }
}
