
/* JADX INFO: loaded from: CWV.class */
public class CWV {
    static java.lang.String mapValue(java.util.Map<java.lang.String, java.lang.String> map) {
        return map.get("k");
    }

    static int mapSize(java.util.Map<?, ?> map) {
        return map.size();
    }

    static int setSize(java.util.Set<java.lang.String> set) {
        return set.size();
    }

    static int collectionSize(java.util.Collection<java.lang.String> collection) {
        return collection.size();
    }

    static int iterableSize(java.lang.Iterable<java.lang.String> iterable) {
        int i = 0;
        java.util.Iterator<java.lang.String> it = iterable.iterator();
        while (it.hasNext()) {
            if (it.next() != null) {
                i++;
            }
        }
        return i;
    }

    static int queueSize(java.util.Queue<java.lang.String> queue) {
        return queue.size();
    }

    static int sortedSetSize(java.util.SortedSet<java.lang.String> sortedSet) {
        return sortedSet.size();
    }

    static java.lang.String sortedMapValue(java.util.SortedMap<java.lang.String, java.lang.String> sortedMap) {
        return sortedMap.get("k");
    }

    static java.lang.String listValue(java.util.List<java.lang.String> list) {
        return list.get(0);
    }

    static int nestedListSize(java.util.List<java.util.List<java.lang.String>> list) {
        return list.size() + list.get(0).size();
    }

    static int dequeSize(java.util.Deque<java.lang.String> deque) {
        return deque.size();
    }

    static int dequeToQueue(java.util.Deque<java.lang.String> deque) {
        return queueSize(deque);
    }

    static int sortedSetToSet(java.util.SortedSet<java.lang.String> sortedSet) {
        return setSize(sortedSet);
    }

    static int navigableSetToSortedSet(java.util.NavigableSet<java.lang.String> navigableSet) {
        return sortedSetSize(navigableSet);
    }

    static java.lang.String navigableMapToSortedMap(java.util.NavigableMap<java.lang.String, java.lang.String> navigableMap) {
        return sortedMapValue(navigableMap);
    }

    static int listToCollection(java.util.List<java.lang.String> list) {
        return collectionSize(list);
    }

    static int setToCollection(java.util.Set<java.lang.String> set) {
        return collectionSize(set);
    }

    static int queueToCollection(java.util.Queue<java.lang.String> queue) {
        return collectionSize(queue);
    }

    static int collectionToIterable(java.util.Collection<java.lang.String> collection) {
        return iterableSize(collection);
    }

    public static void main(java.lang.String[] strArr) {
        java.util.HashMap map = new java.util.HashMap();
        map.put("k", "hashmap");
        java.lang.System.out.println("map:" + mapValue(map));
        java.util.TreeMap treeMap = new java.util.TreeMap();
        treeMap.put("k", "treemap");
        java.lang.System.out.println("treemap:" + mapValue(treeMap));
        java.util.LinkedHashMap linkedHashMap = new java.util.LinkedHashMap();
        linkedHashMap.put("k", "linkedmap");
        java.lang.System.out.println("linkedmap:" + mapValue(linkedHashMap));
        java.util.Hashtable hashtable = new java.util.Hashtable();
        hashtable.put("k", "hashtable");
        java.lang.System.out.println("hashtable:" + mapValue(hashtable));
        java.util.Properties properties = new java.util.Properties();
        properties.setProperty("k", "properties");
        java.lang.System.out.println("properties:" + mapSize(properties));
        java.util.HashSet hashSet = new java.util.HashSet();
        hashSet.add("hashset");
        java.lang.System.out.println("set:" + setSize(hashSet));
        java.util.TreeSet treeSet = new java.util.TreeSet();
        treeSet.add("treeset");
        java.lang.System.out.println("treeset:" + setSize(treeSet));
        java.util.LinkedHashSet linkedHashSet = new java.util.LinkedHashSet();
        linkedHashSet.add("linkedhashset");
        java.lang.System.out.println("linkedhashset:" + setSize(linkedHashSet));
        java.util.ArrayList arrayList = new java.util.ArrayList();
        arrayList.add("coll");
        arrayList.add("two");
        java.lang.System.out.println("collection:" + collectionSize(arrayList));
        java.lang.System.out.println("list:" + listValue(arrayList));
        java.lang.System.out.println("iterable:" + iterableSize(arrayList));
        java.util.LinkedList linkedList = new java.util.LinkedList();
        linkedList.add("linked");
        java.lang.System.out.println("linkedlist:" + listValue(linkedList));
        java.util.Vector vector = new java.util.Vector();
        vector.add("vector");
        java.lang.System.out.println("vector:" + collectionSize(vector));
        java.util.Stack stack = new java.util.Stack();
        stack.push("stack");
        java.lang.System.out.println("stack:" + listValue(stack));
        java.util.ArrayList arrayList2 = new java.util.ArrayList();
        java.util.ArrayList arrayList3 = new java.util.ArrayList();
        arrayList3.add("inner");
        arrayList2.add(arrayList3);
        java.lang.System.out.println("nestedList:" + listValue((java.util.List) arrayList2.get(0)));
        java.util.ArrayList arrayList4 = new java.util.ArrayList();
        arrayList4.add(arrayList3);
        java.lang.System.out.println("nested:" + nestedListSize(arrayList4));
        java.util.ArrayDeque arrayDeque = new java.util.ArrayDeque();
        arrayDeque.add("deque");
        java.lang.System.out.println("deque:" + dequeSize(arrayDeque));
        java.lang.System.out.println("dequeCollection:" + collectionSize(arrayDeque));
        java.util.ArrayList arrayList5 = new java.util.ArrayList();
        arrayList5.add("listinterface");
        arrayList5.add("second");
        java.lang.System.out.println("listToCollection:" + listToCollection(arrayList5));
        java.util.HashSet hashSet2 = new java.util.HashSet();
        hashSet2.add("setinterface");
        java.lang.System.out.println("setToCollection:" + setToCollection(hashSet2));
        java.util.LinkedList linkedList2 = new java.util.LinkedList();
        linkedList2.add("queueinterface");
        java.lang.System.out.println("queueToCollection:" + queueToCollection(linkedList2));
        java.util.ArrayList arrayList6 = new java.util.ArrayList();
        arrayList6.add("collectioninterface");
        arrayList6.add("another");
        java.lang.System.out.println("collectionToIterable:" + collectionToIterable(arrayList6));
        java.util.ArrayDeque arrayDeque2 = new java.util.ArrayDeque();
        arrayDeque2.add("dequeinterface");
        java.lang.System.out.println("dequeToQueue:" + dequeToQueue(arrayDeque2));
        java.util.TreeSet treeSet2 = new java.util.TreeSet();
        treeSet2.add("sortedsetinterface");
        java.lang.System.out.println("sortedSetToSet:" + sortedSetToSet(treeSet2));
        java.util.TreeSet treeSet3 = new java.util.TreeSet();
        treeSet3.add("navigablesetinterface");
        java.lang.System.out.println("navigableSetToSortedSet:" + navigableSetToSortedSet(treeSet3));
        java.util.TreeMap treeMap2 = new java.util.TreeMap();
        treeMap2.put("k", "navigablemapinterface");
        java.lang.System.out.println("navigableMapToSortedMap:" + navigableMapToSortedMap(treeMap2));
    }
}
